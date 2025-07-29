use std::{fmt::Debug, num::NonZeroU8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    Bar,
    Home,
    Board(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionCompressed {
    data: NonZeroU8,
}

impl PositionCompressed {
    pub fn new(data: NonZeroU8) -> Self {
        PositionCompressed { data }
    }

    pub fn get(&self) -> u8 {
        self.data.get()
    }

    /// Creates a Position on the board from an index (0-23).
    pub fn from_index(index: u8) -> Self {
        PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(index + 3) } }
    }

    pub const BAR: Self = PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(1) } };

    pub const HOME: Self = PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(2) } };
}

impl From<Position> for PositionCompressed {
    fn from(pos: Position) -> Self {
        match pos {
            Position::Bar => PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(1) } },
            Position::Home => PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(2) } },
            Position::Board(index) => PositionCompressed { data: unsafe { NonZeroU8::new_unchecked(index + 3) } },
        }
    }
}

impl From<PositionCompressed> for Position {
    fn from(compressed: PositionCompressed) -> Self {
        match compressed.data.get() {
            1 => Position::Bar,
            2 => Position::Home,
            index if index >= 2 => Position::Board(index - 3),
            _ => panic!("Invalid compressed position"),
        }
    }
}

impl Debug for PositionCompressed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data.get() {
            1 => write!(f, "Bar"),
            2 => write!(f, "Home"),
            index if index >= 3 => write!(f, "Board({})", index - 3),
            _ => write!(f, "Invalid PositionCompressed"),
        }
    }
}