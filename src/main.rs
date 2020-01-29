use std::vec;

mod card;
mod comb;
mod game;

use crate::game::*;





fn main() {
    let mut g = Game::new(vec![0, 1]).unwrap();
    dbg!(&g);
    dbg!(&g.make_step(g.get_stepping_player(), Step::GetCard));
    dbg!(&g.make_step(g.get_stepping_player(), Step::GiveComb(g.get_player_cards(g.get_stepping_player()))));
    dbg!(&g.make_step(g.get_stepping_player(), Step::TransComb(g.get_player_cards(g.get_stepping_player()))));
    dbg!(&g.make_step(g.get_stepping_player(), Step::GetComb));
    dbg!(&g);
}