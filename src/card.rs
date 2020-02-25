use serde::de;
use serde::de::{Deserializer, Visitor};
use serde::ser::SerializeTuple;
use serde::ser::Serializer;
use std::fmt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
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

impl<'de> serde::de::Deserialize<'de> for CardRank {
    fn deserialize<D>(deserializer: D) -> Result<CardRank, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CardRankVisitor;
        const FIELDS: &'static [&'static str] = &[
            "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
        ];

        impl<'de> Visitor<'de> for CardRankVisitor {
            type Value = CardRank;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("2 3 4 5 6 7 8 9 10 J Q K A")
            }

            fn visit_str<E>(self, value: &str) -> Result<CardRank, E>
            where
                E: de::Error,
            {
                match value {
                    "2" => Ok(CardRank::Two),
                    "3" => Ok(CardRank::Three),
                    "4" => Ok(CardRank::Four),
                    "5" => Ok(CardRank::Five),
                    "6" => Ok(CardRank::Six),
                    "7" => Ok(CardRank::Seven),
                    "8" => Ok(CardRank::Eight),
                    "9" => Ok(CardRank::Nine),
                    "10" => Ok(CardRank::Ten),
                    "J" => Ok(CardRank::Jack),
                    "Q" => Ok(CardRank::Queen),
                    "K" => Ok(CardRank::King),
                    "A" => Ok(CardRank::Ace),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(CardRankVisitor)
    }
}

impl serde::ser::Serialize for CardRank {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CardRank::Two => serializer.serialize_unit_variant("CardRank", 0, "2"),
            CardRank::Three => serializer.serialize_unit_variant("CardRank", 1, "3"),
            CardRank::Four => serializer.serialize_unit_variant("CardRank", 2, "4"),
            CardRank::Five => serializer.serialize_unit_variant("CardRank", 3, "5"),
            CardRank::Six => serializer.serialize_unit_variant("CardRank", 4, "6"),
            CardRank::Seven => serializer.serialize_unit_variant("CardRank", 5, "7"),
            CardRank::Eight => serializer.serialize_unit_variant("CardRank", 6, "8"),
            CardRank::Nine => serializer.serialize_unit_variant("CardRank", 7, "9"),
            CardRank::Ten => serializer.serialize_unit_variant("CardRank", 8, "10"),
            CardRank::Jack => serializer.serialize_unit_variant("CardRank", 9, "J"),
            CardRank::Queen => serializer.serialize_unit_variant("CardRank", 10, "Q"),
            CardRank::King => serializer.serialize_unit_variant("CardRank", 11, "K"),
            CardRank::Ace => serializer.serialize_unit_variant("CardRank", 12, "A"),
        }
    }
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
    CardRank::Ace,
];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum CardSuit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

impl<'de> serde::de::Deserialize<'de> for CardSuit {
    fn deserialize<D>(deserializer: D) -> Result<CardSuit, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CardSuitVisitor;
        const FIELDS: &'static [&'static str] = &["♠", "♣", "♦", "♥"];

        impl<'de> Visitor<'de> for CardSuitVisitor {
            type Value = CardSuit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("♠ ♣ ♦ ♥")
            }

            fn visit_str<E>(self, value: &str) -> Result<CardSuit, E>
            where
                E: de::Error,
            {
                match value {
                    "♠" => Ok(CardSuit::Spades),
                    "♣" => Ok(CardSuit::Clubs),
                    "♦" => Ok(CardSuit::Diamonds),
                    "♥" => Ok(CardSuit::Hearts),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(CardSuitVisitor)
    }
}

impl serde::ser::Serialize for CardSuit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            CardSuit::Spades => serializer.serialize_unit_variant("CardSuit", 0, "♠"),
            CardSuit::Clubs => serializer.serialize_unit_variant("CardSuit", 1, "♣"),
            CardSuit::Diamonds => serializer.serialize_unit_variant("CardSuit", 2, "♦"),
            CardSuit::Hearts => serializer.serialize_unit_variant("CardSuit", 3, "♥"),
        }
    }
}

pub const CARD_SUITS: [CardSuit; 4] = [
    CardSuit::Spades,
    CardSuit::Clubs,
    CardSuit::Diamonds,
    CardSuit::Hearts,
];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Card {
    pub rank: CardRank,
    pub suit: CardSuit,
}

impl serde::ser::Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element(&self.rank)?;
        seq.serialize_element(&self.suit)?;
        seq.end()
    }
}

impl<'de> serde::de::Deserialize<'de> for Card {
    fn deserialize<D>(deserializer: D) -> Result<Card, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CardVisitor;

        impl<'de> Visitor<'de> for CardVisitor {
            type Value = Card;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("[rack, suit]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Card, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let rank = match seq.next_element() {
                    Ok(Some(rank)) => rank,
                    _ => {
                        return Err(serde::de::Error::missing_field("rank"));
                    }
                };

                let suit = match seq.next_element() {
                    Ok(Some(suit)) => suit,
                    _ => {
                        return Err(serde::de::Error::missing_field("suit"));
                    }
                };

                Ok(Card { rank, suit })
            }
        }

        deserializer.deserialize_seq(CardVisitor)
    }
}

#[test]
fn card_deserialize_test() {
    assert_eq!(
        serde_json::from_str::<Card>(
            &serde_json::to_string(&Card {
                rank: CardRank::Queen,
                suit: CardSuit::Spades
            })
            .unwrap()
        )
        .unwrap(),
        Card {
            rank: CardRank::Queen,
            suit: CardSuit::Spades
        }
    );
}

pub const NUMBER_OF_CARDS: usize = 52;

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<std::cmp::Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}
