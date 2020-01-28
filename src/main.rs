use std::vec;
use std::collections::HashSet;

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
    HighestCard,
    Pair,
    TwoPairs,
    Set,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
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
        if Comb::is_straight_flush(cards) {
            Some(CombRank::StraightFlush)
        } else if Comb::is_four_of_a_kind(cards) {
            Some(CombRank::FourOfAKind)
        } else if Comb::is_full_house(cards) {
            Some(CombRank::FullHouse)
        } else if Comb::is_flush(cards) {
            Some(CombRank::Flush)
        } else if Comb::is_straight(cards) {
            Some(CombRank::Straight)
        } else if Comb::is_set(cards) {
            Some(CombRank::Set)
        } else if Comb::is_two_pairs(cards) {
            Some(CombRank::TwoPairs)
        } else if Comb::is_pair(cards) {
            Some(CombRank::Pair)
        } else if Comb::is_highest_card(cards) {
            Some(CombRank::HighestCard)
        } else {
            None
        }
    }

    fn is_straight_flush(cards: &HashSet<Card>) -> bool {
        if cards.len() == 5 {
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
                            return true;
                        }
                    } else {
                        c = 0
                    }

                }
            }           
        }
        false
    }

    fn is_xy_of_a_kind(cards: &HashSet<Card>, x: usize, y: usize) -> bool {
        if cards.len() == x + y {
            for i in CardRanks.iter() {
                for j in CardRanks.iter() {
                    if i != j && !(y == 0 && *j != CardRanks[0]) {
                        let (mut ci, mut cj) = (0, 0);
                        for k in cards {
                            if k.rank == *i {
                                ci += 1
                            } else if k.rank == *j {
                                cj += 1
                            } 
                            if ci >= x && cj >= y {
                                return true
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn is_x_of_a_kind(cards: &HashSet<Card>, x: usize) -> bool {
        Comb::is_xy_of_a_kind(cards, x, 0)
    }

    fn is_four_of_a_kind(cards: &HashSet<Card>) -> bool {
        Comb::is_x_of_a_kind(cards, 4)
    }


    fn is_full_house(cards: &HashSet<Card>) -> bool {
        Comb::is_xy_of_a_kind(cards, 3, 2)
    }

    fn is_set(cards: &HashSet<Card>) -> bool {
        Comb::is_x_of_a_kind(cards, 3)
    }

    fn is_two_pairs(cards: &HashSet<Card>) -> bool {
        Comb::is_xy_of_a_kind(cards, 2, 2)
    }

    fn is_pair(cards: &HashSet<Card>) -> bool {
        Comb::is_x_of_a_kind(cards, 2)
    }

    fn is_highest_card(cards: &HashSet<Card>) -> bool {
        Comb::is_x_of_a_kind(cards, 1)
    }

    fn is_flush(cards: &HashSet<Card>) -> bool {
        if cards.len() == 5 {
            for i in CardSuits.iter() {
                let mut c = 0;
                for j in cards {
                    if j.suit == *i {
                        c += 1;
                        if c >= 5 {
                            return true;
                        }
                    }
                } 
            }
        }
        false
    }

    fn is_straight(cards: &HashSet<Card>) -> bool {
        if cards.len() == 5 {
            let mut v = vec![false; CardRanks.len()];
            for i in cards {
                v[i.rank as usize] = true;
            }
            let mut c = 0;
            for i in v {
                if i {
                    c += 1;
                    if c == 5 {
                        return true;
                    }
                } else {
                    c = 0;
                }
            }
        }
        false
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
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::StraightFlush);
}

#[test]
fn comb_test_four_of_a_kind() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FourOfAKind);   
}

#[test]
fn comb_test_full_house() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Diamonds}
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FullHouse);   
}

#[test]
fn comb_test_flush() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Nine, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Hearts},
        Card {rank: CardRank::Queen, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Flush);
}

#[test]
fn comb_test_straight() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ten, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Spades},
        Card {rank: CardRank::Queen, suit: CardSuit::Diamonds},
        Card {rank: CardRank::King, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Straight);
}

#[test]
fn comb_test_set() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Diamonds},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Set);
}

#[test]
fn comb_test_two_pairs() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Diamonds}
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::TwoPairs);
}

#[test]
fn comb_test_pair() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Pair);
}

#[test]
fn comb_test_highest_card() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::HighestCard);    
}

#[test]
fn comb_test_nothing() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).is_none(), true);    
}

struct Player {
    id: usize,
    cards: HashSet<Card>,
}


struct Board {
    comb: Comb,
    cards: HashSet<Card>,
}

enum State {
    Active(usize, Board),
    Passive(usize),
    Init(usize),
}


struct Game {
    players: Vec<Player>,
    state: State,
}

fn main() {

}