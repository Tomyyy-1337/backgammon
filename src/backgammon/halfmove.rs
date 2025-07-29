use std::fmt::Debug;

use crate::backgammon::{Position, PositionCompressed};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfMove {
    from: PositionCompressed,
    to: PositionCompressed,
}

impl HalfMove {
    pub fn from_compressed(from: PositionCompressed, to: PositionCompressed) -> Self {
        HalfMove { from, to }
    }

    pub fn from_position(from: Position, to: Position) -> Self {
        HalfMove {
            from: PositionCompressed::from(from),
            to: PositionCompressed::from(to),
        }
    }

    pub fn from(&self) -> &PositionCompressed {
        &self.from
    }

    pub fn to(&self) -> &PositionCompressed {
        &self.to
    }
}

impl Debug for HalfMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.from, self.to)
    }
}