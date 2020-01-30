use std::collections::{HashSet, HashMap};

use rand::{thread_rng, Rng};


use crate::card::*;
use crate::comb::*;

#[derive(PartialEq, Eq, Debug)]
struct Player {
    id: usize,
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

#[derive(Clone, Debug)]
pub struct Board {
    pub comb: Comb,
    pub cards: HashSet<Card>,
}

#[derive(Clone, Debug)]
pub enum State {
    Active(usize, Board),
    Passive(usize),
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
                cards.push(Card {rank: *rank, suit: *suit});
            }
        }
        thread_rng().shuffle(&mut cards);

        Deck {cards}
    }

    pub fn get_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }


    pub fn size(&self) -> usize {
        self.cards.len()
    }
}

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
    players_map: HashMap<usize, usize>,
    deck: Deck,
    state: State,
}

#[derive(Debug)]
pub enum StepError {
    InvalidPlayerID,
    InvalidStepType,
    InvalidCards,
    InvalidComb,
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            StepError::InvalidPlayerID => write!(f, "You can't make step now"),
            StepError::InvalidStepType => write!(f, "You can't use this type of step now"),
            StepError::InvalidCards    => write!(f, "You haven't these cards to make step"),
            StepError::InvalidComb     => write!(f, "Your cards aren't poker combination (or it less strong, than sombination on board)"),
        }
    }
}

impl std::error::Error for StepError {
    fn description(&self) -> &str {
        match &self {
            StepError::InvalidPlayerID => "You can't make step now",
            StepError::InvalidStepType => "You can't use this type of step now",
            StepError::InvalidCards    => "You haven't these cards to make step",
            StepError::InvalidComb     => "Your cards aren't poker combination (or it less strong, than sombination on board)",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Game {
    pub fn new(players_ids: Vec<usize>) -> Option<Game> {
        if players_ids.len() < 52 / 5 {
            let mut players = players_ids.iter().map(|id| Player {id: *id, cards: HashSet::<Card>::new()}).collect::<Vec<_>>();
            thread_rng().shuffle(&mut players);
            let players_map = players.iter().enumerate().map(|x| (x.1.id, x.0)).collect();
            let players_next = (0..(players.len())).map(|x| (x + 1) % players.len()).collect();
            let players_prev = (0..(players.len())).map(|x| (x + players.len() - 1) % players.len()).collect();
            let mut deck = Deck::new();
            for player in players.iter_mut() {
                for _ in 0..5 {
                    player.cards.insert(deck.get_card().unwrap());
                }
            }

            let state = State::Passive(Game::player_min(&players));
            

            Some(Game {players, players_prev, players_next, players_map, deck, state})
        } else {
            None
        }   
    }

    pub fn make_step(&mut self, pid: usize, step: Step) -> Result<(), StepError> {
        match self.state.clone() {
            State::Passive(player) => {
                if self.players_map[&pid] != player {
                    Err(StepError::InvalidPlayerID)
                } else {
                    match step {
                        Step::GetComb | Step::TransComb(_) => Err(StepError::InvalidStepType),
                        Step::GetCard => {
                            if self.deck.size() > 0 {
                                self.players[player].cards.insert(self.deck.get_card().unwrap());  
                            }
                            self.next_player();
                            Ok(())
                        }
                        Step::GiveComb(cards) => {
                            if cards.is_subset(&self.players[player].cards) {
                                match Comb::new(cards.clone()) {
                                    Some(comb) => {
                                        self.players[player].cards = self.players[player].cards.difference(&cards).map(|x| *x).collect();
                                        if self.deck.size() == 0 && self.players[player].cards.len() == 0 {
                                            self.kick_player(pid);
                                        }
                                        self.state = State::Active(player, Board {cards, comb});
                                        self.next_player();
                                        Ok(())
                                    },
                                    None => Err(StepError::InvalidCards)
                                }
                            } else {
                                Err(StepError::InvalidCards)
                            }
                        }
                    }
                }
            }
            State::Active(player, board) => {
                if self.players_map[&pid] != player {
                    Err(StepError::InvalidPlayerID)
                } else {
                    match step {
                        Step::GetCard | Step::GiveComb(_) => Err(StepError::InvalidStepType),
                        Step::TransComb(comb) => {
                            let a = self.players[player].cards.intersection(&comb).collect::<Vec<_>>().len();
                            if a > 0 {
                                if a + board.comb.cards.intersection(&comb).collect::<Vec<_>>().len() < comb.len() {
                                    Err(StepError::InvalidCards)
                                } else {
                                    match Comb::new(comb.clone()) {
                                        None => Err(StepError::InvalidComb),
                                        Some(new_comb) => {
                                            if new_comb > board.comb {
                                                self.players[player].cards = self.players[player].cards.difference(&comb).map(|x| *x).collect();
                                                if self.deck.size() == 0 && self.players[player].cards.len() == 0 {
                                                    self.kick_player(pid);
                                                }
                                                let new_board = Board {cards: board.cards.union(&comb).map(|x| *x).collect(), comb: new_comb};
                                                self.state = State::Active(player, new_board);
                                                self.next_player();
                                                Ok(())
                                            } else {
                                                Err(StepError::InvalidComb)
                                            } 
                                        }
                                    }
                                }
                            } else {
                                Err(StepError::InvalidCards)
                            }
                        }
                        Step::GetComb => {
                            self.players[player].cards = self.players[player].cards.union(&board.cards).map(|x| *x).collect();
                            self.state = State::Passive(player);
                            self.next_player();
                            Ok(())
                        }
                    }
                }
            }
        }
    }

    pub fn kick_player(&mut self, pid: usize) {
        let player = self.players_map[&pid];
        self.players_next[self.players_prev[player]] = self.players_next[player];
        self.players_prev[self.players_next[player]] = self.players_prev[player];
        self.players_next[player] = player; 
    }

    pub fn get_stepping_player(&self) -> usize {
        match &self.state {
            State::Passive(player) => self.players[*player].id,
            State::Active(player, _) => self.players[*player].id,
        }
    }

    pub fn get_player_cards(&self, pid: usize) -> HashSet<Card> {
        self.players[self.players_map[&pid]].cards.clone()
    }

    pub fn get_deck_size(&self) -> usize {
        self.deck.size()
    }

    pub fn is_player_kicked(&self, pid: usize) -> bool {
        self.players_next[self.players_map[&pid]] == self.players_map[&pid]
    }

    pub fn game_winner(&self) -> Option<usize> {
        let player = self.players_map[&self.get_stepping_player()];
        if self.players_next[player] == player {
            Some(self.get_stepping_player())
        } else {
            None
        }
    }

    pub fn get_state_cards(&self) -> State {
        self.state.clone()
    }


    fn next_player(&mut self) {
        self.state = match self.state.clone() {
            State::Passive(player) => State::Passive(self.players_next[player]),
            State::Active(player, board) => State::Active(self.players_next[player], board),
        };
    }

    fn player_min(players: &Vec<Player>) -> usize {
        let mut mini = 0;
        for i in 1..players.len() {
            if players[i] < players[mini] {
                mini = i;
            }
        }
        return mini;
    }
}
