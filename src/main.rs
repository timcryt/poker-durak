use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rand::{thread_rng, Rng};

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rouille;

#[macro_use]
extern crate log;

extern crate env_logger;

use rouille::content_encoding::apply;
use rouille::input;
use rouille::websocket;
use rouille::Response;

mod card;
mod comb;
mod game;

use crate::card::*;
use crate::game::*;

const HEARTBIT_INTERVAL_SECS: u64 = 15;
const TIMEOUT_SECS: u64 = 300;
const PLAYING_ACTIVITY_WAIT_MILLIS: u64 = 200;
const WS_CLOSED_WAIT_SECS: u64 = 5;
const WS_UPDATE_MILLIS: u64 = 100;
const WS_PREWAIT_MILLIS: u64 = 10;
const REFRESH_DURATION_MILLIS: u64 = 250;

struct GamePool {
    players: HashSet<usize>,
    players_channels: HashMap<usize, GameChannelClient>,
    players_time: HashMap<usize, Option<std::time::SystemTime>>,
    waiting_players: HashSet<usize>,
    on_delete: HashMap<usize, Option<GameChannelClient>>,
    counter: usize,
    playing: usize,
}

fn get_sid(request: &rouille::Request) -> Option<usize> {
    if let Some((_, val)) = input::cookies(&request).find(|&(n, _)| n == "sid") {
        match val.trim().parse::<usize>() {
            Ok(sid) => Some(sid),
            _ => None,
        }
    } else {
        None
    }
}

fn data_by_url(url: &str) -> &'static str {
    match url {
        "/" | "/stat" | "/about" | "/winner" | "/loser" => "text/html",
        "/game_script" => "text/javascript",
        "/game_font" => "font/ttf",
        "/favicon.ico" => "image/png",
        url if url.ends_with(".css") => "text/css",
        url if url.ends_with(".html") => "text/html",
        url if url.ends_with(".js") => "text/javascript",
        url if url.ends_with(".ttf") => "font/ttf",
        _ => "text/plain",
    }
}

fn router<'a>(url: &'a str) -> &'a str {
    match url {
        "/" => "/index.html",
        "/stat" => "/stat.html",
        "/about" => "/about.html",
        "/winner" => "/winner.html",
        "/loser" => "/loser.html",
        url => url,
    }
}

fn main() {
    let mut args = args();
    args.next();
    let addr = match args.next() {
        Some(arg) => arg,
        None => "127.0.0.1:8000".to_string(),
    };

    env_logger::init();

    let game_pool = Arc::new(Mutex::new(GamePool {
        players: HashSet::new(),
        players_channels: HashMap::new(),
        players_time: HashMap::new(),
        waiting_players: HashSet::new(),
        on_delete: HashMap::new(),
        counter: 0,
        playing: 0,
    }));

    let addr_clone = addr.clone();

    rouille::start_server(&addr, move |request| {
        router!(request,
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

                apply(request, resp)
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
                    websocket_handling_thread(websocket.recv().unwrap(), game_pool, sid);
                });

                response
            },

            (GET) (/{_any: String}) => {
                let url = request.url();

                match File::open("static".to_string() + router(&url)) {
                    Ok(mut file) => {
                        info!("GET {}", url);
                        let mut data = Vec::new();
                        file.read_to_end(&mut data).unwrap();

                        let all_games = game_pool.lock().unwrap().counter;
                        let now_games = game_pool.lock().unwrap().playing;

                        match String::from_utf8(data.clone()) {
                            Ok(data) =>
                                apply(request, Response::from_data(data_by_url(&url), data
                                    .replace("{host}", &addr_clone)
                                    .replace("{HEARTBIT_INTERVAL}", &HEARTBIT_INTERVAL_SECS.to_string())
                                    .replace("{all_games}", &all_games.to_string())
                                    .replace("{now_games}", &now_games.to_string())
                                )),
                            Err(_) => {
                                apply(request, Response::from_data(data_by_url(&url), data))
                            }
                        }

                    }
                    Err(_) => {
                        warn!("GET {} 404", url);
                        apply(request, rouille::Response::from_file("text/html", File::open("static/404.html").unwrap()).with_status_code(404))
                    }
                }

            },
            _ => {
                warn!("{} {} 404", request.method(), request.url());
                apply(request, rouille::Response::from_file("text/html", File::open("static/404.html").unwrap()).with_status_code(404))
            }
        )
    });
}

fn websocket_next(
    mut websocket: websocket::Websocket,
) -> Option<(websocket::Websocket, Option<websocket::Message>)> {
    let run_flag = Arc::new(Mutex::new(false));
    let run_flag_clone = Arc::clone(&run_flag);

    let child = thread::spawn(move || {
        let msg = websocket.next();
        let mut run_flag = run_flag_clone.lock().unwrap();
        *run_flag = true;
        Some((websocket, msg))
    });

    let now = SystemTime::now();
    sleep(Duration::from_millis(WS_PREWAIT_MILLIS));
    while now.elapsed().unwrap() < Duration::from_secs(HEARTBIT_INTERVAL_SECS) {
        let run_flag = *Arc::clone(&run_flag).lock().unwrap();
        match run_flag {
            true => {
                return child.join().ok().flatten();
            }
            false => (),
        }
        sleep(Duration::from_millis(WS_UPDATE_MILLIS));
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
    YouMadeStep(State, HashSet<Card>, usize, usize),
    StepError(StepError),
    JsonError,
    GameWinner,
    GameLoser,
}

#[derive(Deserialize)]
enum JsonRequest {
    Ping,
    MakeStep(Step),
    Exit,
}

fn player_init(
    game_pool: Arc<Mutex<GamePool>>,
    pid: usize,
) -> (Arc<Mutex<GamePool>>, bool, Option<GameChannelClient>) {
    sleep(Duration::from_millis(PLAYING_ACTIVITY_WAIT_MILLIS));

    if game_pool.lock().unwrap().on_delete.contains_key(&pid) {
        info!("PLAYER {} is restoring", pid);
        let restr_game = game_pool.lock().unwrap().on_delete.remove(&pid).unwrap();
        (game_pool, false, restr_game)
    } else if game_pool.lock().unwrap().players.contains(&pid) {
        if game_pool.lock().unwrap().on_delete.contains_key(&pid) {
            info!("PLAYER {} is restoring", pid);
            let restr_game = game_pool.lock().unwrap().on_delete.remove(&pid).unwrap();
            (game_pool, false, restr_game)
        } else {
            (game_pool, true, None)
        }
    } else {
        game_pool.lock().unwrap().waiting_players.insert(pid);
        info!("PLAYER {} registrated!", pid);
        (game_pool, false, None)
    }
}

fn game_exit(
    game_pool: Arc<Mutex<GamePool>>,
    game: Option<GameChannelClient>,
    websocket: Option<websocket::Websocket>,
    ws_end_success: Option<bool>,
    pid: usize,
) {
    let game_pool = Arc::clone(&game_pool);
    thread::spawn(move || {
        game_pool.lock().unwrap().on_delete.insert(pid, game);

        if ws_end_success == None {
            game_pool.lock().unwrap().waiting_players.remove(&pid);
        } else if ws_end_success == Some(false) {
            info!("PLAYER {} disconnected", pid);
            sleep(Duration::from_secs(WS_CLOSED_WAIT_SECS));
        }

        let mut game_pool = game_pool.lock().unwrap();

        if game_pool.on_delete.contains_key(&pid) {
            if ws_end_success != None {
                let mut game = game_pool.on_delete.remove(&pid).unwrap().unwrap();
                info!("PLAYER {} is exiting!", pid);
                game.kick_player(pid);
                if game.game_winner() == Some(pid) {
                    if let Some(mut websocket) = websocket {
                        websocket
                            .send_text(&serde_json::to_string(&JsonResponse::GameWinner).unwrap())
                            .ok();
                    };
                } else {
                    if let Some(mut websocket) = websocket {
                        websocket
                            .send_text(&serde_json::to_string(&JsonResponse::GameLoser).unwrap())
                            .ok();
                    };
                }

                if game.exit(pid) {
                    game_pool.playing -= 1;
                }

                if game_pool.players.contains(&pid) {
                    game_pool.players.remove(&pid);
                    game_pool.players_channels.remove(&pid);
                    game_pool.players_time.remove(&pid);
                }
            }
            game_pool.on_delete.remove(&pid);

            info!("PLAYER {} exited!", pid);
        }
    });
}

fn game_create(game_pool: Arc<Mutex<GamePool>>) {
    let mut game_pool = game_pool.lock().unwrap();
    let players = game_pool
        .waiting_players
        .iter()
        .copied()
        .collect::<Vec<_>>();
    game_pool.counter += 1;
    let counter = game_pool.counter;
    game_pool.playing += 1;

    info!("GAME {} created", counter);

    let mut now_playing = Vec::new();

    for player in players {
        game_pool.players.insert(player);
        let (serv, clnt) = new_game_channel();
        now_playing.push((player, serv));
        game_pool.players_channels.insert(player, clnt);
        game_pool.players_time.insert(player, None);
    }
    game_pool.waiting_players.clear();

    let counter: usize = game_pool.counter;
    thread::spawn(move || game_worker(now_playing, counter));
}

fn wait_game(
    mut websocket: websocket::Websocket,
    game_pool: Arc<Mutex<GamePool>>,
    pid: usize,
) -> Option<websocket::Websocket> {
    loop {
        if game_pool.lock().unwrap().players.contains(&pid) {
            break;
        }

        let ans = websocket_next(websocket);

        websocket = match ans {
            None => {
                game_exit(game_pool, None, None, None, pid);
                return None;
            }
            Some((websocket, None)) => {
                game_exit(game_pool, None, Some(websocket), None, pid);
                return None;
            }
            Some((mut websocket, message)) => {
                if let websocket::Message::Text(txt) = message.unwrap() {
                    if let Ok(req) = serde_json::from_str::<JsonRequest>(&txt) {
                        match req {
                            JsonRequest::Ping => {
                                websocket
                                    .send_text(&serde_json::to_string(&JsonResponse::Pong).unwrap())
                                    .ok();
                            }
                            _ => (),
                        }
                    }
                }
                websocket
            }
        };
    }
    Some(websocket)
}

fn websocket_handling_thread(
    mut websocket: websocket::Websocket,
    game_pool: Arc<Mutex<GamePool>>,
    pid: usize,
) {
    let (game_pool, is_ret, restr_game) = player_init(game_pool, pid);
    if is_ret {
        websocket
            .send_text(&serde_json::to_string(&JsonResponse::YouArePlaying).unwrap())
            .ok();
        info!("PLAYER {} is playing from another socket", pid);
        return;
    }

    websocket
        .send_text(&serde_json::to_string(&JsonResponse::ID(pid)).unwrap())
        .ok();

    let mut game = if let Some(game) = restr_game {
        game
    } else if game_pool.lock().unwrap().waiting_players.len() >= 2 {
        game_create(Arc::clone(&game_pool));
        game_pool
            .lock()
            .unwrap()
            .players_channels
            .remove(&pid)
            .unwrap()
    } else {
        websocket = match wait_game(websocket, game_pool.clone(), pid) {
            Some(websocket) => websocket,
            None => return,
        };
        game_pool
            .lock()
            .unwrap()
            .players_channels
            .remove(&pid)
            .unwrap()
    };

    info!("PLAYER {} is playing!", pid);
    websocket
        .send_text(
            &serde_json::to_string(&JsonResponse::YourCards(
                game.get_player_cards(pid),
                game.get_deck_size(),
            ))
            .unwrap(),
        )
        .ok();

    let mut your_turn_new = true;
    let mut ws_end_success = false;

    let mut stepping_time: Option<SystemTime> = {
        let mut game_pool = game_pool.lock().unwrap();
        match game_pool.players_time.remove(&pid) {
            Some(time) => time,
            None => None,
        }
    };

    let mut websocket = Some(websocket);
    let mut last_refresh = SystemTime::now();

    loop {
        match websocket_next(websocket.unwrap()) {
            None => {
                websocket = None;
                break;
            }
            Some((ws, None)) => {
                websocket = Some(ws);
                break;
            }
            Some((mut ws, Some(message))) => {
                if last_refresh.elapsed().unwrap() > Duration::from_millis(REFRESH_DURATION_MILLIS)
                {
                    let stepping_player = game.get_stepping_player();
                    if stepping_player == pid && your_turn_new {
                        if stepping_time.is_none() {
                            stepping_time = Some(SystemTime::now());
                        }
                        let time_elapsed = stepping_time.unwrap().elapsed().unwrap().as_secs();

                        ws.send_text(
                            &serde_json::to_string(&JsonResponse::YourTurn(
                                game.get_state_cards(),
                                game.get_player_cards(pid),
                                game.get_deck_size(),
                                game.players_decks()[0],
                                TIMEOUT_SECS - time_elapsed,
                            ))
                            .unwrap(),
                        )
                        .ok();
                        your_turn_new = false;
                    } else if stepping_player == pid
                        && stepping_time.is_some()
                        && stepping_time.unwrap().elapsed().unwrap()
                            > Duration::from_secs(TIMEOUT_SECS)
                    {
                        ws_end_success = true;
                        websocket = Some(ws);
                        break;
                    }

                    if let Some(_) = game.game_winner() {
                        ws_end_success = true;
                        websocket = Some(ws);
                        break;
                    }

                    last_refresh = SystemTime::now();
                }

                match message {
                    websocket::Message::Text(txt) => {
                        if txt != "\"Ping\"" {
                            info!("PLAYER From {} request {}", pid, txt);
                        }

                        let json_response = match serde_json::from_str(&txt) {
                            Ok(json_request) => match json_request {
                                JsonRequest::Ping => JsonResponse::Pong,
                                JsonRequest::MakeStep(step) => match game.make_step(pid, step) {
                                    Ok(()) => {
                                        your_turn_new = true;
                                        stepping_time = None;
                                        if game.is_player_kicked(pid) {
                                            ws_end_success = true;
                                            websocket = Some(ws);
                                            break;
                                        } else {
                                            JsonResponse::YouMadeStep(
                                                game.get_state_cards(),
                                                game.get_player_cards(pid),
                                                game.get_deck_size(),
                                                game.get_player_cards(game.get_stepping_player())
                                                    .len(),
                                            )
                                        }
                                    }
                                    Err(e) => JsonResponse::StepError(e),
                                },
                                JsonRequest::Exit => {
                                    game.kick_player(pid);
                                    ws_end_success = true;
                                    JsonResponse::GameLoser
                                }
                            },
                            Err(_) => JsonResponse::JsonError,
                        };

                        match &json_response {
                            JsonResponse::Pong => (),
                            _ => {
                                info!(
                                    "PLAYER Response {} to {}",
                                    serde_json::to_string(&json_response).unwrap(),
                                    pid
                                );
                            }
                        }

                        ws.send_text(&serde_json::to_string(&json_response).unwrap())
                            .unwrap();

                        if ws_end_success {
                            websocket = Some(ws);
                            break;
                        }
                    }

                    _ => {
                        warn!("PLAYER Unknown message from a websocket {}", pid);
                    }
                }
                websocket = Some(ws);
            }
        }
    }

    if !ws_end_success {
        game_pool
            .lock()
            .unwrap()
            .players_time
            .insert(pid, stepping_time);
    }

    game_exit(game_pool, Some(game), websocket, Some(ws_end_success), pid);
}
