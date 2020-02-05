use std::collections::{HashMap, HashSet};
use std::env::args;
use std::sync::{Mutex, Arc};
use std::time::{Duration, SystemTime};
use std::thread;
use std::thread::sleep;

use rand::{thread_rng, Rng};

use serde::{Serialize, Deserialize};

#[macro_use]
extern crate rouille;

use rouille::websocket;
use rouille::Response;

mod game;
mod comb;
mod card;

use crate::game::*;
use crate::card::*;

struct GamePool {
    games: HashMap<usize, Game>,
    players: HashMap<usize, usize>,
    rev_players: HashMap<usize, HashSet<usize>>,
    waiting_players: HashSet<usize>,
    counter: usize,
}

fn main() {
    let mut args = args();
    args.next();
    let addr = match args.next() {
        Some(arg) => arg,
        None => "127.0.0.1:8000".to_string()
    };

    println!("Now listening on {}", addr);
    let game_pool = Arc::new(Mutex::new(GamePool{
        games: HashMap::new(),
        players: HashMap::new(),
        rev_players: HashMap::new(),
        waiting_players: HashSet::new(),
        counter: 0,
    }));

    rouille::start_server(&addr, move |request| {
        router!(request,
            (GET) (/) => {
                Response::from_file("text/html", std::fs::File::open("static/index.html").unwrap())
            },

            (GET) (/game) => {
                Response::from_file("text/html", std::fs::File::open("static/game.html").unwrap())
            },

            (GET) (/game_script) => {
                Response::from_file("text/javascript", std::fs::File::open("static/game.js").unwrap())
            },

            (GET) (/about) => {
                Response::from_file("text/html", std::fs::File::open("static/about.html").unwrap())
            },

            (GET) (/game_winner) => {
                Response::from_file("text/html", std::fs::File::open("static/winner.html").unwrap())
            },

            (GET) (/game_loser) => {
                Response::from_file("text/html", std::fs::File::open("static/loser.html").unwrap())
            },

            (GET) (/ws) => {
                let (response, websocket) = try_or_400!(websocket::start(&request, Some("echo")));
                let game_pool = Arc::clone(&game_pool);

                thread::spawn(move || {
                    let ws = Arc::new(Mutex::new(websocket.recv().unwrap()));
                    websocket_handling_thread(ws, game_pool);
                });

                response
            },

            (GET) (/stat) => {
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

            _ => rouille::Response::from_file("text/html", std::fs::File::open("static/404.html").unwrap()).with_status_code(404)
        )
    });
}



fn websocket_next(websocket: &Arc<Mutex<websocket::Websocket>>) -> Option<websocket::Message> {
    const HEARTBIT_INTERVAL: u64 = 15;

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
    YourCards(HashSet<Card>, usize, u64),
    YourTurn(State, HashSet<Card>, usize),
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

fn websocket_handling_thread(websocket: Arc<Mutex<websocket::Websocket>>, game_pool: Arc<Mutex<GamePool>>) {
    let pid = thread_rng().gen();
    
    const TIMEOUT_SECS: u64 = 300;


    game_pool.lock().unwrap().waiting_players.insert(pid);


    println!("Player {} registrated!", pid);
    websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::ID(pid)).unwrap()).ok();


    if game_pool.lock().unwrap().waiting_players.len() >= 2 {
        let mut game_pool = game_pool.lock().unwrap();
        let players = game_pool.waiting_players.iter().map(|x| *x).collect::<Vec<_>>();
        game_pool.counter += 1;
        let counter = game_pool.counter;
        game_pool.rev_players.insert(counter, players.iter().map(|x| *x).collect());
        game_pool.games.insert(counter, Game::new(players.clone()).unwrap());
        for player in players {
            game_pool.players.insert(player, counter);
        }
        game_pool.waiting_players.clear();
    } else {
        loop {
            let message = websocket_next(&websocket);
            if message == None {
                println!("Player {} is exiting!", pid);
                game_pool.lock().unwrap().waiting_players.remove(&pid);
                println!("Player {} exited!", pid);
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
                sleep(Duration::from_millis(1000));
                if game_pool.lock().unwrap().players.contains_key(&pid) {
                    break
                }
            }
        }
    }

    let gid = game_pool.lock().unwrap().players[&pid];

    println!("Player {} is playing!", pid);
    {
        let mut game_pool = game_pool.lock().unwrap();
        let game_id = game_pool.players[&pid];
        let game = game_pool.games.get_mut(&game_id).unwrap();
        websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourCards(
            game.get_player_cards(pid),
            game.get_deck_size(),
            TIMEOUT_SECS,
        )).unwrap()).ok();

    }

    let mut your_turn_new = true;
    let mut turn_time = SystemTime::now();

    while let Some(message) = websocket_next(&websocket) {
        {
            let mut game_pool = game_pool.lock().unwrap();
            let game_id = game_pool.players[&pid];
            let game = game_pool.games.get_mut(&game_id).unwrap();
            if game.get_stepping_player() == pid && your_turn_new {
                turn_time = SystemTime::now();
                websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourTurn(
                    game.get_state_cards(),
                    game.get_player_cards(pid),
                    game.get_deck_size(),
                )).unwrap()).ok(); 
                your_turn_new = false; 
            } else if game.get_stepping_player() == pid && turn_time.elapsed().unwrap() > Duration::from_secs(TIMEOUT_SECS) {
                break;
            }
    
        }

        match message {
            websocket::Message::Text(txt) => {

                if txt != "\"Ping\"" {
                    println!("From player {} request {}", pid, txt);
                }

                let json_response = match serde_json::from_str(&txt) {
                    Ok(json_request) => match json_request {
                        JsonRequest::Ping => JsonResponse::Pong,
                        JsonRequest::MakeStep(step) => {
                            let mut game_pool = game_pool.lock().unwrap();
                            let game_id = game_pool.players[&pid];
                            let game = game_pool.games.get_mut(&game_id).unwrap();
                            match game.make_step(pid, step) {
                                Ok(()) => {
                                    your_turn_new = true;
                                    if game.is_player_kicked(pid) {
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
                    _ => {println!("From server response {} to {}", serde_json::to_string(&json_response).unwrap(), pid);}
                }

                websocket.send_text(&serde_json::to_string(&json_response).unwrap()).unwrap();

                {
                    let mut game_pool = game_pool.lock().unwrap();
                    let game_id = game_pool.players[&pid];
                    let game = game_pool.games.get_mut(&game_id).unwrap();
                    if let Some(_) = game.game_winner() {
                        break;
                    }
                }

            },
            _ => {
                println!("received unknown message from a websocket {}", pid);
            },
        }
    }

    {
        let mut game_pool = game_pool.lock().unwrap();
        let game_id = game_pool.players[&pid];
        let game = game_pool.games.get_mut(&game_id).unwrap();
        game.kick_player(pid);
        if game.game_winner() == Some(pid) {
            if let Ok(mut websocket) = websocket.try_lock() {websocket.send_text(&serde_json::to_string(&JsonResponse::GameWinner).unwrap()).ok();}; 
        } else {
            if let Ok(mut websocket) = websocket.try_lock() {websocket.send_text(&serde_json::to_string(&JsonResponse::GameLoser).unwrap()).ok();};   
        }
    }

    println!("Player {} is exiting!", pid);
    {
        let mut game_pool = game_pool.lock().unwrap();
        if game_pool.players.contains_key(&pid) {
            let player_game = game_pool.players[&pid];
            game_pool.games.get_mut(&player_game).unwrap().kick_player(pid);
            game_pool.players.remove(&pid);
        }
        game_pool.rev_players.get_mut(&gid).unwrap().remove(&pid);
        if game_pool.rev_players[&gid].len() == 0 {
            game_pool.games.remove(&gid);
        }
    }
    println!("Player {} exited!", pid);

}