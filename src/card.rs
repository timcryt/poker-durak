use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CardRank {
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


pub const CARD_RANKS: [CardRank; 13] = [
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CardSuit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

pub const CARD_SUITS: [CardSuit; 4] = [CardSuit::Spades, CardSuit::Clubs, CardSuit::Diamonds, CardSuit::Hearts];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Card {
    pub rank: CardRank,
    pub suit: CardSuit,
}

pub const NUMBER_OF_CARDS: usize = 52;

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<std::cmp::Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}