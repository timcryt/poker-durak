use std::collections::HashSet;
use std::vec;

mod test;

use crate::card::*;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comb {
    pub cards: HashSet<Card>,
    #[serde(skip_serializing)]
    rank: CombRank,
}

impl PartialEq for Comb {
    fn eq(&self, other: &Comb) -> bool {
        self.rank == other.rank
    }
}
impl Eq for Comb {}
impl PartialOrd for Comb {
    fn partial_cmp(&self, other: &Comb) -> Option<std::cmp::Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}
impl Ord for Comb {
    fn cmp(&self, other: &Comb) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl Comb {
    pub fn new(cards: HashSet<Card>) -> Option<Comb> {
        let rank = Comb::get_rank(&cards);
        match rank {
            Some(rank) => Some(Comb { cards, rank }),
            None => None,
        }
    }

    fn get_rank(cards: &HashSet<Card>) -> Option<CombRank> {
        match Comb::is_straight_flush(cards) {
            Some(x) => Some(CombRank::StraightFlush(x)),
            None => match Comb::is_four_of_a_kind(cards) {
                Some(x) => Some(CombRank::FourOfAKind(x)),
                None => match Comb::is_full_house(cards) {
                    Some(x) => Some(CombRank::FullHouse(x)),
                    None => match Comb::is_flush(cards) {
                        Some(x) => Some(CombRank::Flush(x)),
                        None => match Comb::is_straight(cards) {
                            Some(x) => Some(CombRank::Straight(x)),
                            None => match Comb::is_set(cards) {
                                Some(x) => Some(CombRank::Set(x)),
                                None => match Comb::is_two_pairs(cards) {
                                    Some(x) => Some(CombRank::TwoPairs(x)),
                                    None => match Comb::is_pair(cards) {
                                        Some(x) => Some(CombRank::Pair(x)),
                                        None => match Comb::is_highest_card(cards) {
                                            Some(x) => Some(CombRank::HighestCard(x)),
                                            None => None,
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn is_straight_flush(cards: &HashSet<Card>) -> Option<CardRank> {
        if cards.len() == 5 {
            for i in CARD_SUITS.iter() {
                let mut v = vec![false; CARD_RANKS.len() + 1];
                for j in cards.iter().filter(|x| x.suit == *i) {
                    v[j.rank as usize + 1] = true;
                    if j.rank == CardRank::Ace {
                        v[0] = true;
                    }
                }
                let mut c = 0;
                for j in (0..v.len()).rev() {
                    if v[j] {
                        c += 1;
                        if c == 5 {
                            return Some(CARD_RANKS[j + 3]);
                        }
                    } else {
                        c = 0
                    }
                }
            }
            None
        } else {
            None
        }
    }

    fn is_xy_of_a_kind(cards: &HashSet<Card>, x: usize, y: usize) -> Option<(CardRank, CardRank)> {
        if cards.len() == x + y {
            let (x, y) = (x.max(y), x.min(y));

            let mut xrank = None;
            for i in CARD_RANKS.iter().rev() {
                if cards.iter().filter(|x| x.rank == *i).count() >= x {
                    xrank = Some(*i);
                    break;
                }
            }

            xrank
                .map(|xr| {
                    let mut yrank = None;
                    for i in CARD_RANKS.iter().rev() {
                        if *i != xr && cards.iter().filter(|x| x.rank == *i).count() >= y {
                            yrank = Some(*i);
                            break;
                        }
                    }
                    yrank.map(|yr| (xr, yr))
                })
                .flatten()
        } else {
            None
        }
    }

    fn is_x_of_a_kind(cards: &HashSet<Card>, x: usize) -> Option<CardRank> {
        if cards.len() == x {
            for i in CARD_RANKS.iter().rev() {
                if cards.iter().filter(|x| x.rank == *i).count() >= x {
                    return Some(*i);
                }
            }
            None
        } else {
            None
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
            let mut m = None;
            for i in CARD_SUITS.iter() {
                let x = cards
                    .iter()
                    .filter_map(|x| if x.suit == *i { None } else { Some(x.rank) });

                if x.clone().count() >= 5 {
                    let mx = x.max().unwrap();
                    m = match m {
                        Some(x) if x > mx => m,
                        _ => Some(mx),
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
            let mut v = [false; CARD_RANKS.len() + 1];
            for i in cards {
                v[i.rank as usize + 1] = true;
                if i.rank == CardRank::Ace {
                    v[0] = true;
                }
            }
            let mut c = 0;
            for i in (0..v.len()).rev() {
                if v[i] {
                    c += 1;
                    if c == 5 {
                        return Some(CARD_RANKS[i + 3]);
                    }
                } else {
                    c = 0;
                }
            }
            None
        } else {
            None
        }
    }
}
