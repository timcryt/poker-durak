use std::io::stdin;
use std::io::prelude::*;
use std::collections::HashSet;

mod game;
mod comb;
mod card;

use crate::game::*;
use crate::card::*;

fn get_int(prompt: &str, min: isize, max: isize) -> isize {
    loop {
        print!("{}", prompt);
        std::io::stdout().flush().unwrap();
        let mut t = String::new();
        stdin().read_line(&mut t).unwrap();

        match t.trim().parse() {
            Ok(n) if n >= min && n <= max => return n,
            _ => (),
        }
    }
}

fn get_card() -> Option<Card> {


    loop {
        let mut t = String::new();
        stdin().read_line(&mut t).unwrap();

        if t.trim() == "" {
            return None;
        }
        let t = t.to_lowercase();

        let t = t.trim().split(" ").collect::<Vec<_>>();
        if t.len() == 2 {
            let (r, s) = (t[0], t[1]);
            let r = match r {
                "двойка" => Some(CardRank::Two),
                "тройка" => Some(CardRank::Three),
                "четвёрка" => Some(CardRank::Four),
                "пятёрка" => Some(CardRank::Five),
                "шестёрка" => Some(CardRank::Six),
                "семёрка" => Some(CardRank::Seven),
                "восьмёрка" => Some(CardRank::Eight),
                "девятка" => Some(CardRank::Nine),
                "десятка" => Some(CardRank::Ten),
                "валет" => Some(CardRank::Jack),
                "дама" => Some(CardRank::Queen),
                "король" => Some(CardRank::King),
                "туз" => Some(CardRank::Ace),
                x => {
                    println!("Неизвестное достоинство {}", x);
                    continue;
                }
            }.unwrap();

            let s = match s {
                "пик" => Some(CardSuit::Spades),
                "треф" | "крестей" => Some(CardSuit::Clubs),
                "бубей" => Some(CardSuit::Diamonds),
                "червей" => Some(CardSuit::Hearts),
                x => {
                    println!("Неизвестная масть {}", x);
                    continue;
                }
            }.unwrap();

            return Some(Card {rank: r, suit: s});
            
        }
    } 
}

fn print_cards(cards: &HashSet<Card>) {
    for card in cards {
        let r = match card.rank {
            CardRank::Two => "Двойка",
            CardRank::Three => "Тройка",
            CardRank::Four => "Четвёрка",
            CardRank::Five => "Пятёрка",
            CardRank::Six => "Шестёрка",
            CardRank::Seven => "Семёрка",
            CardRank::Eight => "Восьмёрка",
            CardRank::Nine => "Девятка",
            CardRank::Ten => "Десятка",
            CardRank::Jack => "Валет",
            CardRank::Queen => "Дама",
            CardRank::King => "Король",
            CardRank::Ace => "Туз",
        };
        let s = match card.suit  {
            CardSuit::Spades => "пик",
            CardSuit::Clubs => "крестей",
            CardSuit::Diamonds => "бубей",
            CardSuit::Hearts => "червей",
        };
        println!("\t{} {}", r, s);
    }
}

fn get_comb() -> HashSet<Card> {
    println!("Перечислите карты, которые хотите выложить в виде <достоинство> <масть>");
    println!("В конце введите пустую строку");
    let mut cards = HashSet::new();
    loop {
        match get_card() {
            Some(card) => {
                cards.insert(card);
                ()
            }
            None => break,
        }
    }
    cards
}

fn make_step(game: &mut Game) -> usize {
    let player = game.get_stepping_player();
    println!("Игрок {}", player);  
    match game.get_state_cards() {
        State::Passive(_) => {
            println!("Против вас нет комбинации");
            println!("Ваши карты: ");
            print_cards(&game.get_player_cards(player));
            println!("Ваши действия:");
            println!("\t1. Взять карту");
            println!("\t2. Выложить комбинацию");
            loop {
                match get_int("Введите команду: ", 1, 2) {
                    1 => match game.make_step(player, Step::GetCard) {
                        Ok(()) => break,
                        Err(e) => println!("Ошибка: {}", e),
                    }
                    2 => match game.make_step(player, Step::GiveComb(get_comb())){
                        Ok(()) => break,
                        Err(e) => println!("Ошибка: {}", e)   
                    }
                    _ => ()
                }
            }
        }
        State::Active(_, board) => {
            println!("Против вас есть комбинация:");
            print_cards(&board.comb.cards);
            println!("На доске также есть карты:");
            print_cards(&board.cards.clone().difference(&board.comb.cards).map(|x| *x).collect());
            println!("Ваши карты:");
            print_cards(&game.get_player_cards(player));
            println!("Ваши действия");
            println!("\t1. Перевести комбинация");
            println!("\t2. Взять комбинацию");
            loop {
                match get_int("Введите команду: ", 1, 2) {
                    1 => match game.make_step(player, Step::TransComb(get_comb())) {
                        Ok(()) => break,
                        Err(e) => println!("Ошибка: {}", e),
                    }
                    2 => match game.make_step(player, Step::GetComb) {
                        Ok(()) => break,
                        Err(e) => println!("Ошибка: {}", e)   
                    }
                    _ => ()
                }
            }            
        }
    }

    if game.is_player_kicked(player) {
        println!("Вы завершили игру!");
        1
    } else {
        0
    }
}

fn main() {
    let mut players = get_int("Число игроков: ", 2, 10) as usize;
    let mut game: Game = Game::new((1..=players).collect()).unwrap();
    while players > 0 {
        for _ in 0..players {
            players -= make_step(&mut game);
        }
    }
}