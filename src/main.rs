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

            (GET) (/ws) => {
                let (response, websocket) = try_or_400!(websocket::start(&request, Some("echo")));
                let game_pool = Arc::clone(&game_pool);

                thread::spawn(move || {
                    let ws = Arc::new(Mutex::new(websocket.recv().unwrap()));
                    websocket_handling_thread(ws, game_pool);
                });

                response
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
    YourCards(HashSet<Card>, usize),
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


    game_pool.lock().unwrap().waiting_players.insert(pid);
    println!("Player {} registrated!", pid);


    if game_pool.lock().unwrap().waiting_players.len() >= 2 {
        let players = game_pool.lock().unwrap().waiting_players.iter().map(|x| *x).collect::<Vec<_>>();
        game_pool.lock().unwrap().counter += 1;
        let counter = game_pool.lock().unwrap().counter;
        game_pool.lock().unwrap().games.insert(counter, Game::new(players.clone()).unwrap());
        for player in players {
            game_pool.lock().unwrap().players.insert(player, counter);
        }
        game_pool.lock().unwrap().waiting_players.clear();
    } else {
        loop {
            sleep(Duration::from_millis(1000));
            if game_pool.lock().unwrap().players.contains_key(&pid) {
                break
            }
        }
    }

    println!("Player {} is playing!", pid);
    {
        let mut game_pool = game_pool.lock().unwrap();
        let game_id = game_pool.players[&pid];
        let game = game_pool.games.get_mut(&game_id).unwrap();
        websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourCards(
            game.get_player_cards(pid),
            game.get_deck_size()
        )).unwrap()).ok();

    }

    let mut your_turn_new = true;

    while let Some(message) = websocket_next(&websocket) {
        {
            let mut game_pool = game_pool.lock().unwrap();
            let game_id = game_pool.players[&pid];
            let game = game_pool.games.get_mut(&game_id).unwrap();
            if game.get_stepping_player() == pid && your_turn_new {
                websocket.lock().unwrap().send_text(&serde_json::to_string(&JsonResponse::YourTurn(
                    game.get_state_cards(),
                    game.get_player_cards(pid),
                    game.get_deck_size(),
                )).unwrap()).ok(); 
                your_turn_new = false; 
            }
    
        }

        match message {
            websocket::Message::Text(txt) => {
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
                println!("received unknown message from a websocket");
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
            dbg!(game.game_winner());
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
    }
    println!("Player {} exited!", pid);

}