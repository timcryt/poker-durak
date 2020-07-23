use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

use rand::{thread_rng, Rng};

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rouille;

#[macro_use]
extern crate log;

use rouille::content_encoding::apply;
use rouille::input;
use rouille::websocket;
use rouille::Response;

mod card;
mod comb;
mod game;

use crate::card::*;
use crate::game::*;

const HEARTBIT_INTERVAL: Duration = Duration::from_secs(15);
const TIMEOUT: Duration = Duration::from_secs(300);
const PLAYING_ACTIVITY_WAIT: Duration = Duration::from_millis(200);
const WS_CLOSED_WAIT: Duration = Duration::from_secs(5);
const WS_UPDATE: Duration = Duration::from_millis(100);
const REFRESH_DURATION: Duration = Duration::from_millis(250);
const MAX_MESSAGE_LENGTH: usize = 4096;

struct GamePool {
    players: HashSet<usize>,
    players_channels: HashMap<usize, GameChannelClient>,
    players_time: HashMap<usize, Option<Instant>>,
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
        "/favicon.ico" => "image/png",
        url if url.ends_with(".css") => "text/css; charset=UTF-8",
        url if url.ends_with(".html") => "text/html; charset=UTF-8",
        url if url.ends_with(".js") => "text/javascript; charset=UTF-8",
        url if url.ends_with(".ttf") => "font/ttf",
        _ => "text/plain; charset=UTF-8",
    }
}

fn router(url: &str) -> &str {
    match url {
        "/" => "/index.html",
        "/stat" => "/stat.html",
        "/about" => "/about.html",
        "/winner" => "/winner.html",
        "/loser" => "/loser.html",
        "/game" => "/game.html",
        url => url,
    }
}

fn set_cookies(request: &rouille::Request, resp: Response) -> Response {
    match get_sid(request) {
        Some(sid) => {
            info!("SID {}", sid);
            resp
        }
        None => {
            let sid = thread_rng().gen::<usize>();
            info!("SID NEW {}", sid);
            resp.with_additional_header("Set-Cookie", format!("sid={}; HttpOnly", sid))
        }
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().unwrap();

    let mut args = args();
    args.next();
    let addr = match args.next() {
        Some(arg) => arg,
        None => "127.0.0.1:8000".to_string(),
    };

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

    info!("Listening on {}", addr);

    rouille::start_server(&addr, move |request| {
        router!(request,
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

                        let (all_games, now_games) = {
                            let game_pool = game_pool.lock().unwrap();
                            (game_pool.counter, game_pool.playing)
                        };


                        let resp = match String::from_utf8(data.clone()) {
                            Ok(data) =>
                                apply(request, Response::from_data(data_by_url(router(&url)), data
                                    .replace("{host}", &addr_clone)
                                    .replace("{HEARTBIT_INTERVAL}", &(HEARTBIT_INTERVAL.as_secs().to_string()))
                                    .replace("{all_games}", &all_games.to_string())
                                    .replace("{now_games}", &now_games.to_string())
                                )),
                            Err(_) => {
                                apply(request, Response::from_data(data_by_url(&url), data))
                            }
                        };

                        if router(&url) == "/game.html" {
                            apply(request, set_cookies(&request, resp))
                        } else {
                            resp
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
) -> Option<(websocket::Websocket, websocket::Message)> {
    let (ft, fr) = mpsc::channel();

    thread::spawn(move || {
        let now = Instant::now();
        ft.send(loop {
            match websocket.try_recv() {
                Ok(msg) => {
                    break Some((websocket, msg));
                }
                Err(websocket::WebsocketRecvError::Empty) => (),
                Err(_) => {
                    break None;
                }
            }
            sleep(WS_UPDATE);
            if now.elapsed() > HEARTBIT_INTERVAL {
                break None;
            }
        })
        .unwrap();
    });

    fr.recv().ok()?
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
enum JsonResponse {
    Pong,
    ID(usize),
    YouArePlaying,
    YourCards(HashSet<Card>, usize),
    YourTurn(State, HashSet<Card>, usize, usize, u64),
    YouMadeStep(State, HashSet<Card>, usize, usize),
    StepError(StepError),
    Message(String),
    Sent(Result<(), ()>),
    JsonError,
    GameWinner,
    GameLoser,
}

#[derive(Deserialize)]
enum JsonRequest {
    Ping,
    MakeStep(Step),
    SendMessage(String),
    Exit,
}

fn player_init(game_pool: Arc<Mutex<GamePool>>, pid: usize) -> (bool, Option<GameChannelClient>) {
    sleep(PLAYING_ACTIVITY_WAIT);

    let mut game_pool = game_pool.lock().unwrap();

    if game_pool.on_delete.contains_key(&pid) {
        info!("PLAYER {} is restoring", pid);
        let restr_game = game_pool.on_delete.remove(&pid).unwrap();
        (false, restr_game)
    } else if game_pool.players.contains(&pid) {
        if game_pool.on_delete.contains_key(&pid) {
            info!("PLAYER {} is restoring", pid);
            let restr_game = game_pool.on_delete.remove(&pid).unwrap();
            (false, restr_game)
        } else {
            (true, None)
        }
    } else {
        game_pool.waiting_players.insert(pid);
        info!("PLAYER {} registrated!", pid);
        (false, None)
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
            sleep(WS_CLOSED_WAIT);
        }

        let mut game_pool = game_pool.lock().unwrap();

        if game_pool.on_delete.contains_key(&pid) {
            if ws_end_success != None {
                let mut game = game_pool.on_delete.remove(&pid).unwrap().unwrap();
                info!("PLAYER {} is exiting!", pid);

                game.kick_me();

                if game.game_winner() == Some(pid) {
                    if let Some(mut websocket) = websocket {
                        websocket
                            .send_text(&serde_json::to_string(&JsonResponse::GameWinner).unwrap())
                            .ok();
                    };
                } else if let Some(mut websocket) = websocket {
                    websocket
                        .send_text(&serde_json::to_string(&JsonResponse::GameLoser).unwrap())
                        .ok();
                }

                if game.exit() {
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

    let mut now_playing = HashMap::new();

    let (cltt, srvr) = mpsc::channel();

    for player in players {
        game_pool.players.insert(player);
        let (srvt, cltr) = mpsc::channel();
        now_playing.insert(player, srvt);
        game_pool.players_channels.insert(
            player,
            GameChannelClient(mpsc::Sender::clone(&cltt), cltr, player),
        );
        game_pool.players_time.insert(player, None);
    }
    game_pool.waiting_players.clear();

    let counter: usize = game_pool.counter;
    thread::spawn(move || game_worker(now_playing, srvr, counter));
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
            Some((mut websocket, message)) => {
                if let websocket::Message::Text(txt) = message {
                    if let Ok(req) = serde_json::from_str::<JsonRequest>(&txt) {
                        if let JsonRequest::Ping = req {
                            websocket
                                .send_text(&serde_json::to_string(&JsonResponse::Pong).unwrap())
                                .ok();
                        }
                    }
                }
                websocket
            }
        };
    }
    Some(websocket)
}

fn refresh_time(
    game: &mut GameChannelClient,
    stepping_time: &mut Option<Instant>,
    your_turn_new: &mut bool,
    pid: usize,
) -> Result<Option<String>, ()> {
    let stepping_player = game.get_stepping_player();
    if stepping_player == pid && *your_turn_new {
        if stepping_time.is_none() {
            *stepping_time = Some(Instant::now());
        }
        let time_elapsed = stepping_time.unwrap().elapsed().as_secs();

        let msg = serde_json::to_string(&JsonResponse::YourTurn(
            game.get_state_cards(),
            game.get_my_cards(),
            game.get_deck_size(),
            game.players_decks()[0],
            TIMEOUT.as_secs() - time_elapsed,
        ))
        .unwrap();
        *your_turn_new = false;
        return Ok(Some(msg));
    } else if stepping_player == pid {
        if let Some(stepping_time) = stepping_time {
            if stepping_time.elapsed() > TIMEOUT {
                return Err(());
            }
        }
    }

    if game.game_winner().is_some() {
        return Err(());
    }
    Ok(None)
}

fn websocket_handling_thread(
    mut websocket: websocket::Websocket,
    game_pool: Arc<Mutex<GamePool>>,
    pid: usize,
) {
    let (is_ret, restr_game) = player_init(Arc::clone(&game_pool), pid);
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
        if let Some(x) = game_pool.lock().unwrap().players_channels.remove(&pid) {
            x
        } else {
            return;
        }
    };

    info!("PLAYER {} is playing!", pid);
    websocket
        .send_text(
            &serde_json::to_string(&JsonResponse::YourCards(
                game.get_my_cards(),
                game.get_deck_size(),
            ))
            .unwrap(),
        )
        .ok();

    let mut your_turn_new = true;
    let mut ws_end_success = false;

    let mut stepping_time: Option<Instant> = {
        let mut game_pool = game_pool.lock().unwrap();
        match game_pool.players_time.remove(&pid) {
            Some(time) => time,
            None => None,
        }
    };

    let mut websocket = Some(websocket);
    let mut last_refresh = Instant::now();

    loop {
        match websocket_next(websocket.unwrap()) {
            None => {
                websocket = None;
                break;
            }
            Some((mut ws, message)) => {
                if last_refresh.elapsed() > REFRESH_DURATION {
                    match refresh_time(&mut game, &mut stepping_time, &mut your_turn_new, pid) {
                        Ok(Some(msg)) => {
                            ws.send_text(&msg).ok();
                        }
                        Err(()) => {
                            ws_end_success = true;
                            websocket = Some(ws);
                            break;
                        }
                        _ => (),
                    }

                    last_refresh = Instant::now();
                }

                for msg in game.get_messages() {
                    info!("MESSAGE \"{}\" sent to {}", msg, pid);
                    ws.send_text(&serde_json::to_string(&JsonResponse::Message(msg)).unwrap())
                        .ok();
                }

                if let websocket::Message::Text(txt) = message {
                    if txt != "\"Ping\"" {
                        info!("PLAYER From {} request {}", pid, txt);
                    }

                    let json_response = match serde_json::from_str(&txt) {
                        Ok(json_request) => match json_request {
                            JsonRequest::Ping => JsonResponse::Pong,
                            JsonRequest::MakeStep(step) => match game.make_step(step) {
                                Ok(()) => {
                                    your_turn_new = true;
                                    stepping_time = None;
                                    if game.is_me_kicked() {
                                        ws_end_success = true;
                                        websocket = Some(ws);
                                        break;
                                    } else {
                                        JsonResponse::YouMadeStep(
                                            game.get_state_cards(),
                                            game.get_my_cards(),
                                            game.get_deck_size(),
                                            game.get_another_number_of_cards(
                                                game.get_stepping_player(),
                                            ),
                                        )
                                    }
                                }
                                Err(e) => JsonResponse::StepError(e),
                            },
                            JsonRequest::SendMessage(msg) => {
                                if msg.len() <= MAX_MESSAGE_LENGTH {
                                    game.send_message(msg);
                                    JsonResponse::Sent(Ok(()))
                                } else {
                                    JsonResponse::Sent(Err(()))
                                }
                            }
                            JsonRequest::Exit => {
                                game.kick_me();
                                ws_end_success = true;
                                JsonResponse::GameLoser
                            }
                        },
                        Err(_) => JsonResponse::JsonError,
                    };

                    let resp = serde_json::to_string(&json_response).unwrap();

                    match &json_response {
                        JsonResponse::Pong => (),
                        _ => {
                            info!("PLAYER Response {} to {}", resp, pid);
                        }
                    }

                    ws.send_text(&resp).unwrap();

                    if ws_end_success {
                        websocket = Some(ws);
                        break;
                    }
                } else {
                    warn!("PLAYER Unknown message from a websocket {}", pid);
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
