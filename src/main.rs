use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Mutex, Arc};
use std::time::{Duration, SystemTime};
use std::thread;
use std::thread::sleep;

use rand::{thread_rng, Rng};

use serde::{Serialize, Deserialize};

#[macro_use]
extern crate rouille;

#[macro_use]
extern crate log;

extern crate env_logger;

use rouille::websocket;
use rouille::input;
use rouille::Response;

mod game;
mod comb;
mod card;

use crate::game::*;
use crate::card::*;

const HEARTBIT_INTERVAL: u64 = 15;

struct GamePool {
    games: HashMap<usize, Game>,
    players: HashMap<usize, (usize, Option<std::time::SystemTime>)>,
    rev_players: HashMap<usize, HashSet<usize>>,
    waiting_players: HashSet<usize>,
    on_delete: HashSet<usize>,
    counter: usize,
}

fn get_sid(request: &rouille::Request) -> Option<usize> {
    if let Some((_, val)) = input::cookies(&request).find(|&(n, _)| n == "sid") {
        match val.trim().parse::<usize>() {
            Ok(sid) => {
                Some(sid)
            }
            _ => {
                None
            }
        }
    } else {
        None
    } 
}

fn main() {
    let mut args = args();
    args.next();
    let addr = match args.next() {
        Some(arg) => arg,
        None => "127.0.0.1:8000".to_string()
    };

    env_logger::init();

    let game_pool = Arc::new(Mutex::new(GamePool{
        games: HashMap::new(),
        players: HashMap::new(),
        rev_players: HashMap::new(),
        waiting_players: HashSet::new(),
        on_delete: HashSet::new(),
        counter: 0,
    }));

    let addr_clone = addr.clone();

    rouille::start_server(&addr, move |request| {
        router!(request,
            (GET) (/) => {
                info!("GET /");
                Response::from_file("text/html", File::open("static/index.html").unwrap())
            },

            (GET) (/game) => {
                info!("GET /game");
                let mut resp = Response::from_file("text/html", File::open("static/game.html").unwrap());
                match get_sid(&request) {
                    Some(sid) => {
                        info!("SID {}", sid);
                    }
                    None => {
                        let sid = thread_rng().gen::<usize>();
                        info!("SID NEW {}", sid);
                        resp = resp.with_additional_header("Set-Cookie", format!("sid={}; HttpOnly", sid));
                    }
                }

                resp
            },

            (GET) (/game_script) => {
                info!("GET /game_script");
                let mut game_script = String::new();
                {
                    let mut script_file = File::open("static/game.js").unwrap();
                    
                    script_file.read_to_string(&mut game_script).unwrap();
                }
            
                Response::from_data("text/javascript", game_script
                    .replace("{host}", &addr_clone)
                    .replace("{HEARTBIT_INTERVAL}", &HEARTBIT_INTERVAL.to_string())
                )
            },

            (GET) (/about) => {
                info!("GET /about");
                Response::from_file("text/html", File::open("static/about.html").unwrap())
            },

            (GET) (/game_winner) => {
                info!("GET /game_winner");
                Response::from_file("text/html", File::open("static/winner.html").unwrap())
            },

            (GET) (/game_loser) => {
                info!("GET /game_loser");
                Response::from_file("text/html", File::open("static/loser.html").unwrap())
            },

            (GET) (/ws) => {
                info!("GET /ws");
                let sid = match get_sid(request) {
                    Some(sid) => {
                        info!("GAME SID {}", sid);
                        sid
                    }
                    None => {
                        let sid = thread_rng().gen::<usize>();
                        info!("GAME SID {} NEW", sid);
                        sid
                    }
                };
                
                let (response, websocket) = try_or_400!(websocket::start(&request, Some("echo")));
                let game_pool = Arc::clone(&game_pool);

                thread::spawn(move || {
                    let ws = Arc::new(Mutex::new(websocket.recv().unwrap()));
                    websocket_handling_thread(ws, game_pool, sid);
                });

                response
            },

            (GET) (/stat) => {
                info!("GET /stat");
                let game_pool = game_pool.lock().unwrap();
                let all_games = game_pool.counter;
                let now_games = game_pool.games.len();
                Response::html(format!(r#"
<html>
    <head>
        <meta charset="UTF-8">
    </head>
    <body>
        <h1>Статистика игры покерный дурак</h1>
        <p>
            Начато игр: {}<br />
            Идёт игр: {}<br />
        </p>
    </body>
</html>
"#, all_games, now_games))
            },

            _ => {
                warn!("{} {} 404", request.method(), request.url());
                rouille::Response::from_file("text/html", File::open("static/404.html").unwrap()).with_status_code(404)
            }
        )
    });
}


fn websocket_next(websocket: &Arc<Mutex<websocket::Websocket>>) -> Option<websocket::Message> {

    let gotten = Arc::new(Mutex::new(None));
    let gotten_clone = Arc::clone(&gotten);
    let run_flag = Arc::new(Mutex::new(false));
    let run_flag_clone = Arc::clone(&run_flag);

    let websocket_clone = Arc::clone(websocket);
    let child = thread::spawn(move || {
        {
            let mut gotten_clone = gotten_clone.lock().unwrap();
            *gotten_clone = websocket_clone.lock().unwrap().next();
            let mut run_flag = run_flag_clone.lock().unwrap();
            *run_flag = true;
        }
    });

    let now = SystemTime::now();
    while now.elapsed().unwrap() < Duration::from_secs(HEARTBIT_INTERVAL) {
        sleep(Duration::from_millis(100));
        let run_flag = *Arc::clone(&run_flag).lock().unwrap();
        match run_flag {
            true => {
                child.join().ok();
                return gotten.lock().unwrap().clone();
            },
            false => (),
        }
    }

    None
}

#[derive(Serialize)]
enum JsonResponse {
    Pong,
    ID(usize),
    YouArePlaying,
    YourCards(HashSet<Card>, usize),
    YourTurn(State, HashSet<Card>, usize, usize, u64),
    YouMadeStep(State, HashSet<Card>, usize),
    StepError(StepError),
    JsonError,
    GameWinner,
    GameLoser,
}

#[derive(Deserialize)]
enum JsonRequest {
    Ping,
    MakeStep(Step)
}

fn player_init(game_pool: Arc<Mutex<GamePool>>, pid: usize) -> (Arc<Mutex<GamePool>>, bool) {
    if game_pool.lock().unwrap().on_delete.contains(&pid) {
        info!("PLAYER {} is restroring", pid);
        game_pool.lock().unwrap().on_delete.remove(&pid);
    } else if game_pool.lock().unwrap().players.contains_key(&pid) {
        const PLAYING_ACTIVITY_WAIT: u64 = 200;
        sleep(Duration::from_millis(PLAYING_ACTIVITY_WAIT));
        if game_pool.lock().unwrap().on_delete.contains(&pid) {
            info!("PLAYER {} is restroring", pid);
            game_pool.lock().unwrap().on_delete.remove(&pid);                
        } else {
            return (game_pool, true);
        }
    } else {
        game_pool.lock().unwrap().waiting_players.insert(pid);
        info!("PLAYER {} registrated!", pid);
    }
    (game_pool, false)
}

fn game_exit(game_pool: Arc<Mutex<GamePool>>, websocket: Arc<Mutex<websocket::Websocket>>, ws_end_success: Option<bool>, pid: usize) {
    let game_pool = Arc::clone(&game_pool);
    thread::spawn(move || {
        const WS_CLOSED_WAIT: u64 = 5; 

        game_pool.lock().unwrap().on_delete.insert(pid);
        if ws_end_success == None {
            game_pool.lock().unwrap().waiting_players.remove(&pid);
        }

        if ws_end_success != Some(true) {
            info!("PLAYER {} disconnected", pid);
            sleep(Duration::from_secs(WS_CLOSED_WAIT));
        }
        
        let mut game_pool = game_pool.lock().unwrap();

        if game_pool.on_delete.contains(&pid) {
            if ws_end_success != None {
                info!("PLAYER {} is exiting!", pid);
                game_pool.on_delete.remove(&pid);

                let gid = game_pool.players[&pid].0;
                let game = game_pool.games.get_mut(&gid).unwrap();
                game.kick_player(pid);
                if game.game_winner() == Some(pid) {
                    if let Ok(mut websocket) = websocket.try_lock() {websocket.send_text(&serde_json::to_string(&JsonResponse::GameWinner).unwrap()).ok();}; 
                } else {
                    if let Ok(mut websocket) = websocket.try_lock() {websocket.send_text(&serde_json::to_string(&JsonResponse::GameLoser).unwrap()).ok();};   
                }

                if game_pool.players.contains_key(&pid) {
                    let player_game = game_pool.players[&pid].0;
                    game_pool.games.get_mut(&player_game).unwrap().kick_player(pid);
                    game_pool.players.remove(&pid);
                }
        
                game_pool.rev_players.get_mut(&gid).unwrap().remove(&pid);
                if game_pool.rev_players[&gid].len() == 0 {
                    game_pool.games.remove(&gid);
                    game_pool.rev_players.remove(&gid);
                    info!("GAME {} deleted", gid);
                }
            }
            info!("PLAYER {} exited!", pid);
        } else if ws_end_success == None {
            game_pool.waiting_players.insert(pid);
        }
    });
}

fn game_create(game_pool: Arc<Mutex<GamePool>>) {
    let mut game_pool = game_pool.lock().unwrap();
    let players = game_pool.waiting_players.iter().map(|x| *x).collect::<Vec<_>>();
    game_pool.counter += 1;
    let counter = game_pool.counter;
    game_pool.rev_players.insert(counter, players.iter().map(|x| *x).collect());
    game_pool.games.insert(counter, Game::new(players.clone()).unwrap());

    info!("GAME {} created", counter);

    for player in players {
        game_pool.players.insert(player, (counter, None));
    }
    game_pool.waiting_players.clear();
}

fn websocket_handling_thread(websocket: Arc<Mutex<websocket::Websocket>>, game_pool: Arc<Mutex<GamePool>>, pid: usize) {
    
    const TIMEOUT_SECS: u64 = 300;

    let (game_pool, is_ret) = player_init(game_pool, pid);
    if is_ret {
        websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YouArePlaying).unwrap()).ok();
        info!("PLAYER {} is playing from another socket", pid);
        return;
    }

    websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::ID(pid)).unwrap()).ok();

    if game_pool.lock().unwrap().waiting_players.len() >= 2 {
        game_create(Arc::clone(&game_pool))
    } else {
        loop {
            let message = websocket_next(&websocket);
            if game_pool.lock().unwrap().players.contains_key(&pid) {
                break
            }
            
            if message == None {
                game_exit(game_pool, websocket, None, pid);
                return;
            } else {
                if let websocket::Message::Text(txt) = message.unwrap() {
                    if let Ok(req) = serde_json::from_str::<JsonRequest>(&txt) {
                        match req {
                            JsonRequest::Ping => {websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::Pong).unwrap()).ok();},
                            _ => ()
                        }
                    }
                }

            }
        }
    }

    info!("PLAYER {} is playing!", pid);
    {
        let mut game_pool = game_pool.lock().unwrap();
        let game_id = game_pool.players[&pid].0;
        let game = game_pool.games.get_mut(&game_id).unwrap();
        websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourCards(
            game.get_player_cards(pid),
            game.get_deck_size(),
        )).unwrap()).ok();

    }

    let mut your_turn_new = true;
    let mut ws_end_success = false;

    while let Some(message) = websocket_next(&websocket) {
        {
            let mut game_pool = game_pool.lock().unwrap();
            let game_id = game_pool.players[&pid].0;
            if game_pool.games[&game_id].get_stepping_player() == pid && your_turn_new {
                if game_pool.players.get_mut(&pid).unwrap().1.is_none() {
                    game_pool.players.get_mut(&pid).unwrap().1 = Some(SystemTime::now());
                }
                let time_elapsed = game_pool.players[&pid].1.unwrap().elapsed().unwrap().as_secs();

                let game = game_pool.games.get(&game_id).unwrap();

                websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourTurn(
                    game.get_state_cards(),
                    game.get_player_cards(pid),
                    game.get_deck_size(),
                    game.players_decks()[0],
                    TIMEOUT_SECS - time_elapsed
                )).unwrap()).ok(); 
                your_turn_new = false; 
            } else if game_pool.games[&game_id].get_stepping_player() == pid && 
                game_pool.players[&pid].1.unwrap().elapsed().unwrap() > Duration::from_secs(TIMEOUT_SECS) {
                ws_end_success = true;
                break;
            }
    
        }

        match message {
            websocket::Message::Text(txt) => {
                if txt != "\"Ping\"" {
                    info!("PLAYER From {} request {}", pid, txt);
                }

                let json_response = match serde_json::from_str(&txt) {
                    Ok(json_request) => match json_request {
                        JsonRequest::Ping => JsonResponse::Pong,
                        JsonRequest::MakeStep(step) => {
                            let mut game_pool = game_pool.lock().unwrap();
                            let game_id = game_pool.players[&pid].0;
                            match game_pool.games.get_mut(&game_id).unwrap().make_step(pid, step) {
                                Ok(()) => {
                                    your_turn_new = true;
                                    game_pool.players.get_mut(&pid).unwrap().1 = None;
                                    let game = game_pool.games.get_mut(&game_id).unwrap();
                                    if game.is_player_kicked(pid) {
                                        ws_end_success = true;
                                        break;
                                    } else {
                                        JsonResponse::YouMadeStep(game.get_state_cards(), game.get_player_cards(pid), game.get_deck_size())
                                    }
                                },
                                Err(e) => JsonResponse::StepError(e),
                            }
                        }
                    }
                    Err(_) => JsonResponse::JsonError
                };
                
                let mut websocket = websocket.lock().unwrap();

                match &json_response {
                    JsonResponse::Pong => (),
                    _ => {info!("PLAYER Response {} to {}", serde_json::to_string(&json_response).unwrap(), pid);}
                }

                websocket.send_text(&serde_json::to_string(&json_response).unwrap()).unwrap();

                {
                    let mut game_pool = game_pool.lock().unwrap();
                    let game_id = game_pool.players[&pid].0;
                    let game = game_pool.games.get_mut(&game_id).unwrap();
                    if let Some(_) = game.game_winner() {
                        ws_end_success = true;
                        break;
                    }
                }

            },
            _ => {
                warn!("PLAYER Unknown message from a websocket {}", pid);
            },
        }
    }
    game_exit(game_pool, websocket, Some(ws_end_success), pid);
}