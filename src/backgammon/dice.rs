use std::{fmt::Debug, mem::MaybeUninit, num::NonZeroU8};
use rand::random_range;

use crate::{backgammon::dice, misc::TinyVec};


/// Represents the dice used in the game of Backgammon. Stores the values of two die and their usage state in 1 byte.
/// 
/// Memory_layout: ABCCCDDD 
/// 
/// C = die2, D = die1 if die1 == die2 and the die has been used 4 times DDD = 111
/// 
/// If die1 == die2, A and B represent a counter how many times the dice was used to move a checker
/// 
/// Else A is 0 if die1 has not been used, B is 0 if die2 has not been used otherwise A and/or B are 1
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Dice {
    data: NonZeroU8
}

impl Dice {
    pub const ALL: [Self; 20] = [
        Dice::from_numbers(1, 1), Dice::from_numbers(1, 2), Dice::from_numbers(1, 3),
        Dice::from_numbers(1, 4), Dice::from_numbers(1, 5), Dice::from_numbers(1, 6),
        Dice::from_numbers(2, 2), Dice::from_numbers(2, 3), Dice::from_numbers(2, 4),
        Dice::from_numbers(2, 5), Dice::from_numbers(2, 6), 
        Dice::from_numbers(3, 3), Dice::from_numbers(3, 4), Dice::from_numbers(3, 5),
        Dice::from_numbers(3, 6), 
        Dice::from_numbers(4, 4), Dice::from_numbers(4, 5), Dice::from_numbers(4, 6),
        Dice::from_numbers(5, 5), 
        Dice::from_numbers(6, 6)
    ];

    pub fn roll() -> Self {
        let die1 = random_range(1..=6);
        let die2 = random_range(1..=6);
        Dice::from_numbers(die1, die2)
    }

    pub fn use_die(&self, die: u8) -> Self {
        let mut dice = *self;
        if die == dice.die1() {
            if dice.is_double() {
                dice.use_double();
            } else {
                dice.use_die1();
            }
        } else {
            dice.use_die2();
        }
        dice
    }

    pub const fn from_numbers(die1: u8, die2: u8) -> Self {
        Dice { data: NonZeroU8::new((die2 << 3) | die1).unwrap() }
    }

    pub fn die1_is_used(&self) -> bool {
        (self.is_double() && self.die_is_used_double()) || 
        (!self.is_double() && self.die_1_is_used_non_double())
    }

    pub fn die2_is_used(&self) -> bool {
        (self.is_double() && self.die_is_used_double()) || 
        (!self.is_double() && self.die_2_is_used_non_double())
    }

    // dont use this function on doubles.
    pub fn die_1_is_used_non_double(&self) -> bool {
        self.data.get() & 0b10000000 == 0b10000000
    }

    // dont use this function on doubles.s
    pub fn die_2_is_used_non_double(&self) -> bool {
        self.data.get() & 0b01000000 == 0b01000000
    }

    // only use this function on doubles.
    pub fn die_is_used_double(&self) -> bool {
        self.data.get() & 0b11111000 == 0b11111000
    }

    // dont use this function on doubles.    
    pub fn use_die1(&mut self) {
        self.data = unsafe { NonZeroU8::new_unchecked(self.data.get() | 0b10000000) };    
    }
    
    pub fn all_used(&self) -> bool {
        (self.is_double() && self.die_is_used_double()) || 
        (!self.is_double() && self.die_1_is_used_non_double() && self.die_2_is_used_non_double())
    }

    // dont use this function on doubles.    
    pub fn use_die2(&mut self) {
        self.data = unsafe { NonZeroU8::new_unchecked(self.data.get() | 0b01000000) };    
    }

    // only use on doubles
    pub fn use_double(&mut self) {
        let current = self.data.get() & 0b11000000;
        if current == 0b11000000 {
            self.data = unsafe { NonZeroU8::new_unchecked(self.data.get() | 0b11111000) };
        } else {
            self.data = unsafe { NonZeroU8::new_unchecked((self.data.get() & 0b00111111) | ((current >> 6) + 1) << 6) };
        }
    }

    // Dont use this function on doubles. It will return a wrong value(7) for fully used doubles.
    pub fn die2(&self) -> u8 {
        (self.data.get() >> 3) & 0x7 
    }

    pub fn die1(&self) -> u8 {
        self.data.get() & 0x7
    }

    pub fn is_double(&self) -> bool {
        self.die1() == self.die2() || self.data.get() & 0b11111000 == 0b11111000
    }

    pub fn availiable_dice(&self) -> TinyVec<u8, 2> {
        if self.is_double() {
            if self.die_is_used_double() {
                TinyVec::new()
            } else {
                TinyVec::from_raw([MaybeUninit::new(self.die1()), MaybeUninit::uninit()], 1)
            }
        } else {
            let mut vec = TinyVec::new();
            if !self.die_1_is_used_non_double() {
                vec.push(self.die1());
            }
            if !self.die_2_is_used_non_double() {
                vec.push(self.die2());
            }
            vec
        }
    }
}

impl Debug for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_double() {
            write!(f, "Double({}): ", self.die1())?;
            if self.die_is_used_double() {
                write!(f, "Used 4 times")?;
            } else {
                write!(f, "Used {} times", (self.data.get() >> 6) + 1)?;
            }
        } else {
            write!(f, "Dice({} | {}): ", self.die1(), self.die2())?;
            if self.die_1_is_used_non_double() {
                write!(f, "Die1 used")?;
            } else {
                write!(f, "Die1 not used")?;
            }
            if self.die_2_is_used_non_double() {
                write!(f, ", Die2 used")?;
            } else {
                write!(f, ", Die2 not used")?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice() {
        for i in 1..=6 {
            for j in 1..=6 {
                let mut dice = Dice::from_numbers(i, j);
                assert_eq!(dice.die1(), i);
                assert_eq!(dice.die2(), j);
                if i == j {
                    assert!(dice.is_double());
                    let mut count = 0;
                    while !dice.die1_is_used() {
                        count += 1;
                        dice.use_double();
                        assert!(dice.is_double());
                    }
                    assert!(dice.die1_is_used());
                    assert!(dice.die2_is_used());
                    assert_eq!(count, 4);
                    assert!(dice.is_double());
                } else {
                    assert!(!dice.is_double());
                    assert!(!dice.die1_is_used());
                    assert!(!dice.die2_is_used());
                    dice.use_die1();
                    assert!(!dice.is_double());
                    assert!(dice.die1_is_used());
                    assert!(!dice.die2_is_used());
                    dice.use_die2();
                    assert!(dice.die2_is_used());
                    assert!(dice.die1_is_used());
                    assert!(!dice.is_double())
                }
            }
        }
    }

    #[test]
    fn test_dice_roll() {
        for _ in 1..=100 {
            let dice = Dice::roll();
            assert!(dice.die1() >= 1 && dice.die1() <= 6);
            assert!(dice.die2() >= 1 && dice.die2() <= 6);
            if dice.is_double() {
                assert_eq!(dice.die1(), dice.die2());
            } else {
                assert_ne!(dice.die1(), dice.die2());
            }
        }
    }
}

