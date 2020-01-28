use std::vec;
use std::collections::{HashSet, HashMap};

use rand::{thread_rng, Rng};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
enum CardRank {
    Two,
    Three,
    Four,
    Five, 
    Six, 
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}


const CardRanks: [CardRank; 13] = [
    CardRank::Two,
    CardRank::Three,
    CardRank::Four,
    CardRank::Five,
    CardRank::Six,
    CardRank::Seven,
    CardRank::Eight,
    CardRank::Nine,
    CardRank::Ten,
    CardRank::Jack,
    CardRank::Queen,
    CardRank::King,
    CardRank::Ace
];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum CardSuit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

const CardSuits: [CardSuit; 4] = [CardSuit::Spades, CardSuit::Clubs, CardSuit::Diamonds, CardSuit::Hearts];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Card {
    rank: CardRank,
    suit: CardSuit,
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<std::cmp::Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum CombRank {
    HighestCard(CardRank),
    Pair(CardRank),
    TwoPairs((CardRank, CardRank)),
    Set(CardRank),
    Straight(CardRank),
    Flush(CardRank),
    FullHouse((CardRank, CardRank)),
    FourOfAKind(CardRank),
    StraightFlush(CardRank),
}

#[derive(Debug)]
struct Comb {
    cards: HashSet<Card>,
    rank: CombRank,
}

impl Comb {
    pub fn new(cards: HashSet<Card>) -> Option<Comb> {
        let rank = Comb::get_rank(&cards);
        match rank {
            Some(rank) => Some(Comb {cards, rank}),
            None => None
        }
    }

    fn get_rank(cards: &HashSet<Card>) -> Option<CombRank> {
        match Comb::is_straight_flush(cards) {
            Some(x) => Some(CombRank::StraightFlush(x)),
            None => 
        match Comb::is_four_of_a_kind(cards) {
            Some(x) => Some(CombRank::FourOfAKind(x)),
            None =>
        match Comb::is_full_house(cards) {
            Some(x) => Some(CombRank::FullHouse(x)),
            None =>
        match Comb::is_flush(cards) {
            Some(x) => Some(CombRank::Flush(x)),
            None =>
        match Comb::is_straight(cards) {
            Some(x) => Some(CombRank::Straight(x)),
            None =>
        match Comb::is_set(cards) {
            Some(x) => Some(CombRank::Set(x)),
            None =>
        match Comb::is_two_pairs(cards) {
            Some(x) => Some(CombRank::TwoPairs(x)),
            None =>
        match Comb::is_pair(cards) {
            Some(x) => Some(CombRank::Pair(x)),
            None =>
        match Comb::is_highest_card(cards) {
            Some(x) => Some(CombRank::HighestCard(x)),
            None => None
        }}}}}}}}}
    }

    fn is_straight_flush(cards: &HashSet<Card>) -> Option<CardRank> {
        if cards.len() == 5 {
            let mut m: Option<CardRank> = None;
            for i in CardSuits.iter() {
                let mut v = vec![false; CardRanks.len()];
                for j in cards {
                    if j.suit == *i {
                        v[j.rank as usize] = true; 
                    }
                }
                let mut c = 0;
                for j in 0..CardRanks.len() {
                    if v[j] {
                        c += 1;
                        if c == 5 {
                            m = match m {
                                None =>
                                    Some(CardRanks[j]),
                                Some(x) =>
                                    if x < CardRanks[j] {
                                        Some(CardRanks[j])
                                    } else {
                                        Some(x)
                                    }
                            }
                        }
                    } else {
                        c = 0
                    }

                }
            }       
            m    
        } else {
            None
        }
    }

    fn is_xy_of_a_kind(cards: &HashSet<Card>, x: usize, y: usize) -> Option<(CardRank, CardRank)> {
        if cards.len() == x + y {
            let mut m: Option<(CardRank, CardRank)> = None;
            for i in CardRanks.iter() {
                for j in CardRanks.iter() {
                    if i < j && !(y == 0 && *i != CardRanks[0]) {
                        let (mut ci, mut cj) = (0, 0);
                        for k in cards {
                            if k.rank == *i {
                                ci += 1
                            } else if k.rank == *j {
                                cj += 1
                            } 
                            if (ci >= x && cj >= y) || (ci >= y && cj >= x) {
                                let t = (*j, *i);
                                m = match m {
                                    None =>
                                        Some(t),
                                    Some(x) =>
                                        if x < t {
                                            Some(t)
                                        } else {
                                            Some(x)
                                        }
                                }
                            }
                        }
                    }
                }
            }
            m
        } else {
            None
        }
    }

    fn is_x_of_a_kind(cards: &HashSet<Card>, x: usize) -> Option<CardRank> {
        match Comb::is_xy_of_a_kind(cards, x, 0) {
            None => None,
            Some((a, b)) => Some(a),
        }
    }

    fn is_four_of_a_kind(cards: &HashSet<Card>) -> Option<CardRank> {
        Comb::is_x_of_a_kind(cards, 4)
    }


    fn is_full_house(cards: &HashSet<Card>) -> Option<(CardRank, CardRank)> {
        Comb::is_xy_of_a_kind(cards, 3, 2)
    }

    fn is_set(cards: &HashSet<Card>) -> Option<CardRank> {
        Comb::is_x_of_a_kind(cards, 3)
    }

    fn is_two_pairs(cards: &HashSet<Card>) -> Option<(CardRank, CardRank)> {
        Comb::is_xy_of_a_kind(cards, 2, 2)
    }

    fn is_pair(cards: &HashSet<Card>) -> Option<CardRank> {
        Comb::is_x_of_a_kind(cards, 2)
    }

    fn is_highest_card(cards: &HashSet<Card>) -> Option<CardRank> {
        Comb::is_x_of_a_kind(cards, 1)
    }

    fn is_flush(cards: &HashSet<Card>) -> Option<CardRank> {
        if cards.len() == 5 {
            let mut m: Option<CardRank> = None;
            for i in CardSuits.iter() {
                let mut c = 0;
                for j in cards {
                    if j.suit == *i {
                        c += 1;
                    }
                } 
                if c >= 5 {
                    for j in cards {
                        if j.suit == *i {
                            m = match m {
                                None =>
                                    Some(j.rank),
                                Some(x) =>
                                    if x < j.rank {
                                        Some(j.rank)
                                    } else {
                                        Some(x)
                                    }
                            }
                        }
                    }
                }
            }
            m
        } else {
            None
        }
    }

    fn is_straight(cards: &HashSet<Card>) -> Option<CardRank> {
        if cards.len() == 5 {
            let mut m: Option<CardRank> = None;
            let mut v = vec![false; CardRanks.len()];
            for i in cards {
                v[i.rank as usize] = true;
            }
            let mut c = 0;
            for i in 0..v.len() {
                if v[i] {
                    c += 1;
                    if c >= 5 {
                        m = Some(CardRanks[i]);
                    }
                } else {
                    c = 0;
                }
            }
            m
        } else {
            None
        }
    }
}

#[test]
fn comb_test_straight_flush() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ten, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Hearts},
        Card {rank: CardRank::Queen, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::StraightFlush(CardRank::Ace));
}

#[test]
fn comb_test_four_of_a_kind() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FourOfAKind(CardRank::Ace));   
}

#[test]
fn comb_test_full_house() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Diamonds}
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FullHouse((CardRank::Ace, CardRank::King)));   
}

#[test]
fn comb_test_flush() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Nine, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Hearts},
        Card {rank: CardRank::Queen, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Flush(CardRank::Ace));
}

#[test]
fn comb_test_straight() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ten, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Spades},
        Card {rank: CardRank::Queen, suit: CardSuit::Diamonds},
        Card {rank: CardRank::King, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Straight(CardRank::Ace));
}

#[test]
fn comb_test_set() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Set(CardRank::Ace));
}

#[test]
fn comb_test_two_pairs() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Diamonds}
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::TwoPairs((CardRank::Ace, CardRank::King)));
}

#[test]
fn comb_test_pair() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Pair(CardRank::Ace));
}

#[test]
fn comb_test_highest_card() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::HighestCard(CardRank::Ace));    
}

#[test]
fn comb_test_nothing() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).is_none(), true);    
}

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

#[derive(Debug)]
struct Board {
    comb: Comb,
    cards: HashSet<Card>,
}

#[derive(Debug)]
enum State {
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
        for rank in CardRanks.iter() {
            for suit in CardSuits.iter() {
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

#[derive(Debug)]
struct Game {
    players: Vec<Player>,
    players_map: HashMap<usize, usize>,
    deck: Deck,
    state: State,
}

impl Game {
    pub fn new(players_ids: Vec<usize>) -> Option<Game> {
        if players_ids.len() < 52 / 5 {
            let mut players = players_ids.iter().map(|id| Player {id: *id, cards: HashSet::<Card>::new()}).collect::<Vec<Player>>();
            thread_rng().shuffle(&mut players);
            let players_map = players.iter().enumerate().map(|x| (x.0, x.1.id)).collect::<HashMap<usize, usize>>();
            let mut deck = Deck::new();
            for player in players.iter_mut() {
                for i in 0..5 {
                    player.cards.insert(deck.get_card().unwrap());
                }
            }

            let state = State::Passive(Game::player_min(&players));
            

            Some(Game {players, players_map, deck, state})
        } else {
            None
        }   
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

fn main() {
    
}