use crate::backgammon::{Board, Player};

pub struct Game {
    board: Board,
    active_player: Player,
    home: [u8; 2],
}