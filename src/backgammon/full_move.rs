use std::{fmt::Debug, ops::Deref};

use crate::{backgammon::HalfMove, misc::TinyVec};

#[derive(Clone, Copy)]
pub struct Move {
    half_moves: TinyVec<HalfMove, 4>,
}

impl Move {
    pub fn new() -> Self {
        Move {
            half_moves: TinyVec::new(),
        }
    }

    pub fn add_half_move(&mut self, half_move: HalfMove) {
        self.half_moves.push(half_move);
    }

    pub fn unordered_equal(&self, other: &Self) -> bool {
        let mut used: u8 = 0;
        for half_move in self.half_moves.iter() {
            match other.half_moves.iter().enumerate().position(|(i,&hm)| hm == *half_move && !used & (1 << i) != 0) {
                Some(index) => used |= 1 << index,
                None => return false,
            }       
        }
        true 
    }
}

impl Deref for Move {
    type Target = TinyVec<HalfMove, 4>;

    fn deref(&self) -> &Self::Target {
        &self.half_moves
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Move: ")?;
        for (i, half_move) in self.half_moves.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", half_move)?;
        }
        Ok(())
    }
}