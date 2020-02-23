use crate::comb::*;
use crate::card::*;

#[test]
fn comb_test_straight_flush() {
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ten, suit: CardSuit::Hearts},
        Card {rank: CardRank::Jack, suit: CardSuit::Hearts},
        Card {rank: CardRank::Queen, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::StraightFlush(CardRank::Ace));
    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Two, suit: CardSuit::Hearts},
        Card {rank: CardRank::Three, suit: CardSuit::Hearts},
        Card {rank: CardRank::Four, suit: CardSuit::Hearts},
        Card {rank: CardRank::Five, suit: CardSuit::Hearts},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::StraightFlush(CardRank::Five));
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
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FullHouse(((CardRank::Ace, CardRank::King), 3)));  
    

    assert_eq!(Comb::new(vec![
        Card {rank: CardRank::Ace, suit: CardSuit::Spades},
        Card {rank: CardRank::Ace, suit: CardSuit::Clubs},
        Card {rank: CardRank::King, suit: CardSuit::Spades},
        Card {rank: CardRank::King, suit: CardSuit::Hearts},
        Card {rank: CardRank::King, suit: CardSuit::Diamonds}
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::FullHouse(((CardRank::Ace, CardRank::King), 2))); 
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
    assert_eq!(Comb::new(vec![    
        Card {rank: CardRank::Two, suit: CardSuit::Hearts},
        Card {rank: CardRank::Three, suit: CardSuit::Spades},
        Card {rank: CardRank::Four, suit: CardSuit::Diamonds},
        Card {rank: CardRank::Five, suit: CardSuit::Clubs},
        Card {rank: CardRank::Ace, suit: CardSuit::Hearts},
        ].into_iter().collect::<HashSet<_>>()).unwrap().rank, CombRank::Straight(CardRank::Five));
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