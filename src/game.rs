use std::collections::{HashMap, HashSet};

use rand::{thread_rng, seq::SliceRandom};

use serde::{Deserialize, Serialize};

use crate::card::*;
use crate::comb::*;

const PLAYERS_CARDS: usize = 5;

type PID = usize;

#[derive(PartialEq, Eq, Debug)]
struct Player {
    id: PID,
    cards: HashSet<Card>,
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Player) -> Option<std::cmp::Ordering> {
        let mut v1 = self.cards.iter().map(|card| card.rank).collect::<Vec<_>>();
        v1.sort();
        let mut v2 = other.cards.iter().map(|card| card.rank).collect::<Vec<_>>();
        v2.sort();
        Some(v1.cmp(&v2))
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub comb: Comb,
    pub cards: HashSet<Card>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Active(Board),
    Passive,
}

#[derive(Debug)]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = Vec::<Card>::new();
        for rank in CARD_RANKS.iter() {
            for suit in CARD_SUITS.iter() {
                cards.push(Card {
                    rank: *rank,
                    suit: *suit,
                });
            }
        }
        cards.shuffle(&mut thread_rng());

        Deck { cards }
    }

    pub fn get_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn get_cards(&mut self, n: usize) -> HashSet<Card> {
        (0..n).filter_map(|_| self.get_card()).collect()
    }

    pub fn size(&self) -> usize {
        self.cards.len()
    }
}

#[derive(Serialize, Deserialize)]
pub enum Step {
    GetCard,
    GiveComb(HashSet<Card>),
    TransComb(HashSet<Card>),
    GetComb,
}

#[derive(Debug)]
pub struct Game {
    players: Vec<Player>,
    players_prev: Vec<usize>,
    players_next: Vec<usize>,
    players_map: HashMap<PID, usize>,
    stepping_player: usize,
    winner: Option<usize>,
    deck: Deck,
    state: State,
}

#[derive(Debug, Serialize)]
pub enum StepError {
    InvalidPID,
    InvalidStepType,
    InvalidCards,
    InvalidComb,
    WeakComb,
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            StepError::InvalidPID => write!(f, "Вы не можете соверщить шаг сейчас"),
            StepError::InvalidStepType => write!(f, "Вы не имеете права делать данный тип шага"),
            StepError::InvalidCards => write!(f, "У вас нет карт, чтобы сделать этот шаг"),
            StepError::InvalidComb => write!(f, "Ваши карты не являются покерной комбинацией"),
            StepError::WeakComb => write!(f, "Ваша комбинация слишком слаба"),
        }
    }
}

impl std::error::Error for StepError {
    fn description(&self) -> &str {
        match &self {
            StepError::InvalidPID => "Вы не можете совершить шаг сейчас",
            StepError::InvalidStepType => "Вы не имеете права делать данный тип шага",
            StepError::InvalidCards => "У вас нет карт, чтобы сделать этот шаг",
            StepError::InvalidComb => "Ваши карты не являются покерной комбинацией",
            StepError::WeakComb => "Ваша комбинация слишком слаба",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Game {
    pub fn new(players_ids: Vec<PID>) -> Option<Game> {
        if players_ids.len() < NUMBER_OF_CARDS / PLAYERS_CARDS {
            let mut players = players_ids
                .iter()
                .map(|id| Player {
                    id: *id,
                    cards: HashSet::<Card>::new(),
                })
                .collect::<Vec<_>>();
            players.shuffle(&mut thread_rng());
            let players_map = players.iter().enumerate().map(|x| (x.1.id, x.0)).collect();

            let (players_next, players_prev): (Vec<_>, Vec<_>) = (0..(players.len()))
                .map(|x| {
                    (
                        (x + 1) % players.len(),
                        (x + players.len() - 1) % players.len(),
                    )
                })
                .unzip();

            let mut deck = Deck::new();
            players.iter_mut().for_each(|player| {
                player.cards = player
                    .cards
                    .union(&deck.get_cards(PLAYERS_CARDS))
                    .copied()
                    .collect()
            });

            let state = State::Passive;
            let stepping_player = Game::player_min(&players);

            Some(Game {
                players,
                stepping_player,
                players_prev,
                players_next,
                players_map,
                winner: None,
                deck,
                state,
            })
        } else {
            None
        }
    }

    fn win_player(&mut self, pid: PID) {
        let player = self.players_map[&pid];

        if self.players_next[player] == self.players_prev[player] && self.winner == None {
            self.winner = Some(player);
        }

        self.players_next[self.players_prev[player]] = self.players_next[player];
        self.players_prev[self.players_next[player]] = self.players_prev[player];
        self.players_next[player] = player;
        if self.get_stepping_player() == pid {
            self.next_player();
        }
    }

    fn next_player(&mut self) {
        let player = self.stepping_player;
        self.stepping_player = self.players_next[player];
    }

    fn player_min(players: &[Player]) -> usize {
        players
            .iter()
            .enumerate()
            .map(|x| (x.1, x.0))
            .min()
            .unwrap()
            .1
    }

    fn cards_for_winners(&mut self) {
        let player = self.get_stepping_player();
        self.next_player();

        while self.get_stepping_player() != player {
            if let Some(card) = self.deck.get_card() {
                let player = self.get_stepping_player();
                self.players[self.players_map[&player]].cards.insert(card);
            }
            self.next_player();
        }
    }

    fn get_cards_for_players(&mut self) {
        let player = self.get_stepping_player();
        let mut f = true;
        while self.get_stepping_player() != player || f {
            self.next_player();
            let player = self.get_stepping_player();
            let number_of_cards = self.players[self.players_map[&player]].cards.len();

            if number_of_cards < 5 {
                self.players[self.players_map[&player]].cards = self.players
                    [self.players_map[&player]]
                    .cards
                    .union(&self.deck.get_cards(5 - number_of_cards))
                    .cloned()
                    .collect();
            }

            f = false;
        }
    }
}

pub trait GameTrait {
    fn make_step(&mut self, pid: PID, step: Step) -> Result<(), StepError>;
    fn players_decks(&self) -> Vec<usize>;
    fn kick_player(&mut self, pid: PID);
    fn get_stepping_player(&self) -> PID;
    fn get_player_cards(&self, pid: PID) -> HashSet<Card>;
    fn get_deck_size(&self) -> usize;
    fn is_player_kicked(&self, pid: PID) -> bool;
    fn game_winner(&self) -> Option<PID>;
    fn get_state_cards(&self) -> State;
}

impl GameTrait for Game {
    fn make_step(&mut self, pid: PID, step: Step) -> Result<(), StepError> {
        let player = self.stepping_player;
        if self.players_map[&pid] != player {
            Err(StepError::InvalidPID)
        } else {
            match self.state.clone() {
                State::Passive => match step {
                    Step::GetComb | Step::TransComb(_) => Err(StepError::InvalidStepType),
                    Step::GetCard => {
                        if self.deck.size() > 0 {
                            self.players[player]
                                .cards
                                .insert(self.deck.get_card().unwrap());
                            self.next_player();
                            Ok(())
                        } else {
                            Err(StepError::InvalidStepType)
                        }
                    }
                    Step::GiveComb(cards) => {
                        if cards.is_subset(&self.players[player].cards) {
                            match Comb::new(cards.clone()) {
                                Some(comb) => {
                                    self.players[player].cards = self.players[player]
                                        .cards
                                        .difference(&cards)
                                        .copied()
                                        .collect();
                                    self.state = State::Active(Board { cards, comb });

                                    if self.deck.size() == 0
                                        && self.players[player].cards.is_empty()
                                    {
                                        self.winner = Some(player);
                                        self.win_player(pid);
                                    } else {
                                        self.next_player();
                                    }

                                    Ok(())
                                }
                                None => Err(StepError::InvalidCards),
                            }
                        } else {
                            Err(StepError::InvalidCards)
                        }
                    }
                },
                State::Active(board) => {
                    let pid = self.players[player].id;
                    match step {
                        Step::GetCard | Step::GiveComb(_) => Err(StepError::InvalidStepType),
                        Step::TransComb(comb) => {
                            let a = self.players[player].cards.intersection(&comb).count();
                            if a > 0 {
                                if a + board.cards.intersection(&comb).count() < comb.len() {
                                    Err(StepError::InvalidCards)
                                } else {
                                    match Comb::new(comb.clone()) {
                                        None => Err(StepError::InvalidComb),
                                        Some(new_comb) => {
                                            if new_comb > board.comb {
                                                self.players[player].cards = self.players[player]
                                                    .cards
                                                    .difference(&comb)
                                                    .copied()
                                                    .collect();
                                                let new_board = Board {
                                                    cards: board
                                                        .cards
                                                        .union(&comb)
                                                        .copied()
                                                        .collect(),
                                                    comb: new_comb,
                                                };
                                                self.state = State::Active(new_board);

                                                if self.deck.size() == 0
                                                    && self.players[player].cards.is_empty()
                                                {
                                                    self.winner = Some(player);
                                                    self.win_player(pid);
                                                } else {
                                                    self.next_player();
                                                }

                                                Ok(())
                                            } else {
                                                Err(StepError::WeakComb)
                                            }
                                        }
                                    }
                                }
                            } else {
                                Err(StepError::InvalidCards)
                            }
                        }
                        Step::GetComb => {
                            self.players[player].cards = self.players[player]
                                .cards
                                .union(&board.comb.cards)
                                .copied()
                                .collect();
                            self.get_cards_for_players();
                            self.cards_for_winners();
                            self.state = State::Passive;
                            self.next_player();
                            Ok(())
                        }
                    }
                }
            }
        }
    }

    fn players_decks(&self) -> Vec<usize> {
        (1..self.players.len())
            .map(|i| {
                self.players[(self.stepping_player + i) % self.players.len()]
                    .cards
                    .len()
            })
            .collect()
    }

    fn kick_player(&mut self, pid: PID) {
        let player = self.players_map[&pid];

        if self.players_next[player] == self.players_prev[player] && self.winner == None {
            self.winner = Some(self.players_next[player]);
        }

        self.players_next[self.players_prev[player]] = self.players_next[player];
        self.players_prev[self.players_next[player]] = self.players_prev[player];
        self.players_next[player] = player;
        if self.get_stepping_player() == pid {
            self.next_player();
        }
    }

    fn get_stepping_player(&self) -> PID {
        self.players[self.stepping_player].id
    }

    fn get_player_cards(&self, pid: PID) -> HashSet<Card> {
        self.players[self.players_map[&pid]].cards.clone()
    }

    fn get_deck_size(&self) -> usize {
        self.deck.size()
    }

    fn is_player_kicked(&self, pid: PID) -> bool {
        self.players_next[self.players_map[&pid]] == self.players_map[&pid]
    }

    fn game_winner(&self) -> Option<PID> {
        match self.winner {
            None => None,
            Some(winner) => Some(self.players[winner].id),
        }
    }

    fn get_state_cards(&self) -> State {
        self.state.clone()
    }
}

pub struct GameChannelServer(
    std::sync::mpsc::Receiver<GameRequest>,
    std::sync::mpsc::Sender<GameResponse>,
);

pub struct GameChannelClient(
    std::sync::mpsc::Sender<GameRequest>,
    std::sync::mpsc::Receiver<GameResponse>,
);

enum GameRequest {
    MakeStep(usize, Step),
    GetPlayersDecks,
    KickPlayer(PID),
    GetSteppingPlayer,
    GetPlayerCards(PID),
    GetDeckSize,
    IsPlayerKicked(PID),
    GetGameWinner,
    GetState,
    Exit(PID),
}

enum GameResponse {
    YouMadeStep(Result<(), StepError>),
    PlayersDecks(Vec<usize>),
    SteppingPlayer(PID),
    YourCards(HashSet<Card>),
    DeckSize(usize),
    PlayerKicked(bool),
    GameWinner(Option<PID>),
    GameState(State),
    Exited(bool),
}

impl GameChannelClient {
    pub fn exit(self, pid: PID) -> bool {
        self.0.send(GameRequest::Exit(pid)).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::Exited(f) => f,
            _ => panic!(),
        }
    }
}

impl GameTrait for GameChannelClient {
    fn make_step(&mut self, pid: PID, step: Step) -> Result<(), StepError> {
        self.0.send(GameRequest::MakeStep(pid, step)).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::YouMadeStep(res) => res,
            _ => panic!(),
        }
    }

    fn players_decks(&self) -> Vec<usize> {
        self.0.send(GameRequest::GetPlayersDecks).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::PlayersDecks(res) => res,
            _ => panic!(),
        }
    }

    fn kick_player(&mut self, pid: PID) {
        self.0.send(GameRequest::KickPlayer(pid)).unwrap();
    }

    fn get_stepping_player(&self) -> usize {
        self.0.send(GameRequest::GetSteppingPlayer).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::SteppingPlayer(pid) => pid,
            _ => panic!(),
        }
    }

    fn get_player_cards(&self, pid: PID) -> HashSet<Card> {
        self.0.send(GameRequest::GetPlayerCards(pid)).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::YourCards(cards) => cards,
            _ => panic!(),
        }
    }

    fn get_deck_size(&self) -> usize {
        self.0.send(GameRequest::GetDeckSize).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::DeckSize(size) => size,
            _ => panic!(),
        }
    }

    fn is_player_kicked(&self, pid: PID) -> bool {
        self.0.send(GameRequest::IsPlayerKicked(pid)).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::PlayerKicked(f) => f,
            _ => panic!(),
        }
    }

    fn game_winner(&self) -> Option<PID> {
        self.0.send(GameRequest::GetGameWinner).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::GameWinner(winner) => winner,
            _ => panic!(),
        }
    }

    fn get_state_cards(&self) -> State {
        self.0.send(GameRequest::GetState).unwrap();
        match self.1.recv().unwrap() {
            GameResponse::GameState(state) => state,
            _ => panic!(),
        }
    }
}

const MANAGER_SLEEP: std::time::Duration = std::time::Duration::from_millis(20);

pub fn game_worker(players: Vec<(PID, GameChannelServer)>, gid: usize) {
    let mut playing = (0..players.len()).map(|_| true).collect::<Vec<_>>();
    let mut count = players.len();
    let mut game = Game::new(players.iter().map(|x| x.0).collect()).unwrap();
    let players = players.into_iter().map(|x| x.1).collect::<Vec<_>>();
    info!("GAME {} started", gid);
    'outer: loop {
        for player in players.iter().enumerate() {
            if playing[player.0] {
                match (player.1).0.try_recv() {
                    Ok(req) => (match req {
                        GameRequest::MakeStep(pid, step) => {
                            Some(GameResponse::YouMadeStep(game.make_step(pid, step)))
                        }
                        GameRequest::GetPlayersDecks => {
                            Some(GameResponse::PlayersDecks(game.players_decks()))
                        }
                        GameRequest::KickPlayer(pid) => {
                            game.kick_player(pid);
                            None
                        }
                        GameRequest::GetSteppingPlayer => {
                            Some(GameResponse::SteppingPlayer(game.get_stepping_player()))
                        }
                        GameRequest::GetPlayerCards(pid) => {
                            Some(GameResponse::YourCards(game.get_player_cards(pid)))
                        }
                        GameRequest::GetDeckSize => {
                            Some(GameResponse::DeckSize(game.get_deck_size()))
                        }
                        GameRequest::IsPlayerKicked(pid) => {
                            Some(GameResponse::PlayerKicked(game.is_player_kicked(pid)))
                        }
                        GameRequest::GetGameWinner => {
                            Some(GameResponse::GameWinner(game.game_winner()))
                        }
                        GameRequest::GetState => {
                            Some(GameResponse::GameState(game.get_state_cards()))
                        }
                        GameRequest::Exit(pid) => {
                            game.kick_player(pid);
                            playing[player.0] = false;
                            count -= 1;
                            if count == 0 {
                                (player.1).1.send(GameResponse::Exited(true)).unwrap();
                                break 'outer;
                            }
                            Some(GameResponse::Exited(false))
                        }
                    })
                    .map_or((), |resp| (player.1).1.send(resp).unwrap()),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        playing[player.0] = false;
                        count -= 1;
                        if count == 0 {
                            break 'outer;
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                }
            }
        }
        std::thread::sleep(MANAGER_SLEEP);
    }
    info!("GAME {} exiting", gid);
}

pub fn new_game_channel() -> (GameChannelServer, GameChannelClient) {
    let (send_1, recv_1) = std::sync::mpsc::channel();
    let (send_2, recv_2) = std::sync::mpsc::channel();
    (
        GameChannelServer(recv_1, send_2),
        GameChannelClient(send_1, recv_2),
    )
}
