use crate::{backgammon::{Dice, HalfMove, Move, Player, PositionCompressed}, misc::TinyVec};

/// Representation of a Backgammon board using 128 bits.
/// The board is spit into two [`u64`] values. The 4 least significant bits
/// represent the number of checkers on the bar for each player. The checkers on
/// the board are represented as 5 bits per position. (4 bits for the number, 1 for the sign).
/// The board always represents the position from the perspective of the active player.
/// The memory layout aims to be compact while allowing fast access for move generation and fast inversion 
/// of the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    board: [u64; 2],
    home: u8,
    active_player: Player,
}

impl Board {

    /// Creates a new board with the default starting position.
    pub fn new() -> Self {
        Board {
            board: [
                // Pos 1 Pos 2 Pos 3 Pos 4 Pos 5 Pos 6 Pos 7 Pos 8 Pos 9 Pos10 Pos11 Pos12 Active_Bar
                0b_00010_00000_00000_00000_00000_10101_00000_10011_00000_00000_00000_00101_0000, 
                // Pos24 Pos23 Pos22 Pos21 Pos20 Pos19 Pos18 Pos17 Pos16 Pos15 Pos14 Pos13 Passive_Bar
                0b_10010_00000_00000_00000_00000_00101_00000_00011_00000_00000_00000_10101_0000
            ],
            home: 0,
            active_player: Player::White,
        }
    }

    /// Creates an empty board with no checkers on it.
    pub fn empty() -> Self {
        Board {
            board: [0, 0],
            home: 0,
            active_player: Player::White,
        }
    }

    /// Return the number of checkers on the bar for the active player.
    pub fn get_active_bar(&self) -> u8 {
        (self.board[0] & 0x000000000000000F) as u8
    }

    /// Set the number of checkers on the bar for the active player.
    pub fn set_active_bar(&mut self, value: u8) {
        self.board[0] = (self.board[0] & !0x000000000000000F) | (value as u64);
    }

    /// Return the number of checkers on the bar for the passive player.
    pub fn get_passive_bar(&self) -> u8 {
        (self.board[1] & 0x000000000000000F) as u8
    }

    /// Set the number of checkers on the bar for the passive player.
    pub fn set_passive_bar(&mut self, value: u8) {
        self.board[1] = (self.board[1] & !0x000000000000000F) | (value as u64);
    }

    /// [`Board`] should only be used for move generation.
    /// Use [`crate::backgammon::Game`] for playing and evaluating the game.
    pub fn get_active_home(&self) -> u8 {
        self.home & 0xF
    }

    pub fn set_active_home(&mut self, value: u8) {
        self.home = (self.home & 0xF0) | (value & 0xF);
    }

    /// [`Board`] should only be used for move generation.
    /// Use [`crate::backgammon::Game`] for playing and evaluating the game.
    pub fn get_passive_home(&self) -> u8 {
        self.home >> 4
    }

    pub fn get_player_on_position(&self, index: u8) -> Option<Player> {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let val = self.board[board_index] >> index_offset;
        match val & 0xF {
            0 => None, 
            _ => match val & 0x10 {
                0x10 => Some(self.active_player), 
                _ => Some(self.active_player.opposite()),
            }
            
        }
    }

    pub fn get_count_on_position(&self, index: u8) -> u8 {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let val = self.board[board_index] >> index_offset;
        (val & 0xF) as u8
    }

    pub fn get_checkers_on_position(&self, index: u8) -> i8 {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let val = self.board[board_index] >> index_offset;
        let abs = (val & 0xF) as i8;
        let sign = (((val & 0x10) >> 4) as i8) * -2 + 1;
        sign * abs
    }

    /// Set a abitrary number of checkers on a position.
    /// Positive values for the active player, negative values for the passive player.
    /// If you want to set the number of checkers for a player, use 
    /// [`Self::set_active_player_checker_on_position`] or 
    /// [`Self::set_passive_player_checker_on_position`] for optimal performance.
    pub fn set_checkers_on_position(&mut self, index: u8, value: i8) {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let mask = 0x1F << index_offset;
        let abs = value.abs() as u64;
        let sign = ((value as u64 >> 7) & 1) << 4; // 0x10 if negative, 0 if positive
        let encoded = (abs | sign) << index_offset;
        self.board[board_index] = (self.board[board_index] & !mask) | encoded;
    }

    pub fn set_active_player_checker_on_position(&mut self, index: u8, value: u8) {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let mask = 0x1F << index_offset;
        self.board[board_index] = (self.board[board_index] & !mask) | (value as u64) << index_offset;
    }

    pub fn set_passive_player_checker_on_position(&mut self, index: u8, value: u8) {
        let index_offset = Self::index_offset(index);
        let board_index = (index / 12) as usize;
        let mask = 0x1F << index_offset;
        self.board[board_index] = (self.board[board_index] & !mask) | ((value as u64) << index_offset) | (0x10 << index_offset);
    }

    const INDEX_OFFSET_LOOKUP: [u8; 24] = [
        59, 54, 49, 44, 39, 34, 29, 24, 19, 14, 9, 4,
        4, 9, 14, 19, 24, 29, 34, 39, 44, 49, 54, 59
    ];

    pub fn index_offset(index: u8) -> u8 {
        Self::INDEX_OFFSET_LOOKUP[index as usize]
    }

    const INVERT_SIGN_MASK: u64 = 0b1000010000100001000010000100001000010000100001000010000100000000;

    pub fn switch_player(&mut self) {
        self.board[0] ^= self.board[1];
        self.board[1] ^= self.board[0];
        self.board[0] ^= self.board[1];
        self.board[0] ^= Self::INVERT_SIGN_MASK;
        self.board[1] ^= Self::INVERT_SIGN_MASK;        
        self.active_player = self.active_player.opposite();
        self.home = (self.home << 4) | self.home >> 4; 
    }

    pub fn active_home_board(&self) -> impl Iterator<Item = i8> {
        (0..6).map(move |i| self.get_checkers_on_position(18 + i as u8))
    }

    pub fn generate_moves(&self, dice: Dice) -> Vec<Move> {
        let mut stack: Vec<(Dice, Board, Move)> = vec![(dice, *self, Move::new())];
        let mut next_stack: Vec<(Dice, Board, Move)> = Vec::new();
        
        let mut best_result_len = 0;
        let mut results = Vec::new();

        loop {
            while let Some((dice, board, previous_moves)) = stack.pop() {
                let previous_moves_len = previous_moves.len();
                if previous_moves_len > best_result_len {
                    results.clear();
                    best_result_len = previous_moves.len();
                    results.push(previous_moves);
                } else if previous_moves_len == best_result_len {
                    results.push(previous_moves);
                } 
                if dice.all_used() {
                    continue;
                }
                
                let half_moves = board.generate_half_moves(dice);
                
                for &(hv, remaining_dice) in half_moves.iter() {
                    let mut board = board.clone();
                    board.make_halfmove_unchecked(&hv);
                    let mut mv = previous_moves;
                    mv.add_half_move(hv);
                    next_stack.push((remaining_dice, board, mv));
                }
            }
            if next_stack.is_empty() {
                break;
            }
            stack.clear();
            for e in next_stack.drain(..) {
                if !stack.iter().any(|(_, _, mv)| mv.unordered_equal(&e.2)) {
                    stack.push(e);
                }
            }
        }
        results
    }

    pub fn active_player_can_bear_off(&self) -> bool {
        let sum = (18..24)
            .map(|i| self.get_checkers_on_position(i))
            .filter(|&x| x > 0)
            .sum::<i8>();

        sum + self.get_active_home() as i8 == 15
    }

    pub fn generate_half_moves(&self, dice: Dice) -> TinyVec<(HalfMove, Dice), 15> {
        let available_dice = dice.availiable_dice();
        let mut half_moves = TinyVec::new();

        if self.get_active_bar() == 0 {
            for &die in available_dice.iter() {
                for i in 0..(24 - die as usize) {
                    if self.get_checkers_on_position(i as u8) > 0 && self.get_checkers_on_position(i as u8 + die) >= -1 {
                        half_moves.push((
                            HalfMove::from_compressed(
                                PositionCompressed::from_index(i as u8),
                                PositionCompressed::from_index(i as u8 + die),
                            ),
                            dice.use_die(die)
                        ));
                    }
                }        
            }
            if self.active_player_can_bear_off() {
                for &die in available_dice.iter() {
                    let indx = 6 - die as usize;
                    if self.get_checkers_on_position(18 + indx as u8) > 0 {
                        half_moves.push((
                            HalfMove::from_compressed(
                                PositionCompressed::from_index(24 - die),
                                PositionCompressed::HOME,
                            ),
                            dice.use_die(die),
                        ));
                    }
                }
                if half_moves.is_empty() {
                    for &die in available_dice.iter() {
                        let indx = 6 - die as usize;
                        for (i,v) in self.active_home_board().skip(indx).enumerate() {
                            if v > 0 {
                                half_moves.push((
                                    HalfMove::from_compressed(
                                        PositionCompressed::from_index(18 + indx as u8 + i as u8),
                                        PositionCompressed::HOME,
                                    ),
                                    dice.use_die(die),
                                ));
                            }
                        }
                    }
                }
            }
        } else {
            for &die in available_dice.iter() {
                if self.get_checkers_on_position(die - 1) >= -1 {
                    half_moves.push((
                        HalfMove::from_compressed(
                            PositionCompressed::BAR,
                            PositionCompressed::from_index(die - 1),
                        ),
                        dice.use_die(die),
                    ));
                }
            }
        }

        half_moves
    }


    pub fn make_halfmove_unchecked(&mut self, half_move: &HalfMove) {
        match half_move.from().get() {
            1 => self.set_active_bar(self.get_active_bar() - 1),
            2 => panic!("Cannot move from home"),
            n => {
                let count = self.get_checkers_on_position(n - 2);
                self.set_checkers_on_position(n - 2, count - 1);
            }
        }
        match half_move.to().get() {
            1 => panic!("Cannot move to bar"),
            2 => self.set_active_home(self.get_active_home() + 1),
            n => {
                let mut count = self.get_checkers_on_position(n - 2);
                if count >= -1 {
                    if count == -1 {
                        self.set_passive_bar(self.get_passive_bar() + 1);
                        count = 0;
                    }
                    self.set_checkers_on_position(n - 2, count + 1);
                }
            }
        }
    }

    pub fn make_move_unchecked(&mut self, full_move: Move) {
        for half_move in full_move.iter() {
            self.make_halfmove_unchecked(half_move);
        }
        self.switch_player();
    }

}