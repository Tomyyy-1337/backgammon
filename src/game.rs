use std::{mem::swap, num::NonZeroU8};

use rand::random_range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

pub enum GameOutcome {
    Win(Player),
    Gammon(Player),
    Backgammon(Player),
    Ongoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Board {
    board: [i8; 24],
    active_bar: u8,
    inactive_bar: u8,
    active_home: u8,
    inactive_home: u8,
    active_player: Player,
}

impl Board {
    pub fn from_whites_perspective(&self) -> Board {
        match self.active_player {
            Player::White => *self,
            Player::Black => {
                let mut board = *self;
                board.invert_board();
                board
            }

        }
    }

    pub fn checkers_on_position(&self, position: u8) -> i8 {
        match self.active_player {
            Player::White => self.board[position as usize],
            Player::Black => -self.board[23 - position as usize],
        }
    }

    pub fn bar(&self, player: Player) -> u8 {
        if self.active_player == player {
            self.active_bar
        } else {
            self.inactive_bar
        }
    }

    pub fn home(&self, player: Player) -> u8 {
        if self.active_player == player {
            self.active_home
        } else {
            self.inactive_home
        }
    }

    pub fn active_player(&self) -> Player {
        self.active_player
    }

    pub fn to_fancy_string(&self) -> String {
        let board = self.from_whites_perspective();
        format!(
"12  11  10   9   8   7  | W |   6   5   4   3   2   1 
{:2}  {:2}  {:2}  {:2}  {:2}  {:2}  | {:1} |  {:2}  {:2}  {:2}  {:2}  {:2}  {:2}
=========================================================
{:2}  {:2}  {:2}  {:2}  {:2}  {:2}  | {:1} |   {:2}  {:2}  {:2}  {:2}  {:2}  {:2}
13  14  15  16  17  18  | B |   19  20  21  22  23  24",
            board.board[11], board.board[10], board.board[9], board.board[8], board.board[7], board.board[6], 
            board.active_bar,
            board.board[5], board.board[4], board.board[3], board.board[2], board.board[1], board.board[0],
            board.board[12], board.board[13], board.board[14], board.board[15], board.board[16], board.board[17],
            board.inactive_bar,
            board.board[18], board.board[19], board.board[20], board.board[21], board.board[22], board.board[23])
    }

    pub fn outcome(&self) -> GameOutcome {
        let active_home_clear = || self.active_home_board().iter().filter(|&&a| a < 0).sum::<i8>() == 0;
        let inactive_home_clear = || self.inactive_home_board().iter().filter(|&&a| a > 0).sum::<i8>() == 0;

        match (self.active_home, self.inactive_home) {
            (15, 0) if self.inactive_bar == 0 && active_home_clear() => GameOutcome::Gammon(self.active_player),
            (15, 0) => GameOutcome::Backgammon(self.active_player),
            (15, _) => GameOutcome::Win(self.active_player),
            (0, 15) if self.active_bar == 0 && inactive_home_clear() => GameOutcome::Gammon(self.active_player.opposite()),
            (0, 15) => GameOutcome::Backgammon(self.active_player.opposite()),
            (_, 15) => GameOutcome::Win(self.active_player.opposite()),               
            _ => GameOutcome::Ongoing,
        }
    }

    pub fn eval_absolute(&self) -> f32 {
        match self.active_player {
            Player::White => self.eval(),
            Player::Black => -self.eval(),
        }
    }

    pub fn eval(&self) -> f32 {
        let mut score = 0;

        match self.outcome() {
            GameOutcome::Win(player) if player == self.active_player => return 1000.0,
            GameOutcome::Win(_) => return -1000.0,
            GameOutcome::Gammon(player) if player == self.active_player => return 2000.0,
            GameOutcome::Gammon(_) => return -2000.0,
            GameOutcome::Backgammon(player) if player == self.active_player => return 3000.0,
            GameOutcome::Backgammon(_) => return -3000.0,
            _ => {}
        }

        for (i, &checker) in self.board.iter().enumerate() {
            if checker > 0 {
                let mult = i as i16 + 1;
                score += checker as i16 * mult.min(19);
            } else if checker < 0 {
                let mult = 24 - i as i16;
                score += checker as i16 * mult.min(19);         
            } 
            if i >= 18 && checker >= 2 {
                score += 1;
            } else if i < 6 && checker <= -2 {
                score -= 1;
            }
        }
        
        score += (self.active_home as i16 - self.inactive_home as i16) * 21;

        score -= (self.active_bar as i16 - self.inactive_bar as i16) * 5;  

        score as f32
    }

    pub fn new() -> Self {
        Board {
            board: [2,0,0,0,0,-5,0,-3,0,0,0,5,-5,0,0,0,3,0,5,0,0,0,0,-2],
            active_bar: 0,
            inactive_bar: 0,
            active_home: 0,
            inactive_home: 0,
            active_player: Player::White,
        }
    }

    pub fn bench() -> Self {
        Board {
            board: [1,-2,-2,1,1,0,1,-1,0,0,0,-2,-1,-1,0,0,0,0,6,-1,-1,-4,2,2],
            active_bar: 1,
            inactive_bar: 0,
            active_home: 0,
            inactive_home: 0,
            active_player: Player::White,
        }
    }

    pub fn captured_value(&self, m: &Move) -> u8 {
        let mut sum = 0;
        for half_move in m.half_moves.iter() {
            let HalfMove { to, ..} = half_move;
            match to.to_enum() {
                PositionEnum::Board(n) if self.board[n as usize] == -1 => sum += n + 1,
                _ => (),
            }
        }

        sum
    }

    pub fn switch_player(&mut self) {
        self.active_player = match self.active_player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        };
        self.invert_board();
    }

    fn invert_board(&mut self) {
        for i in 0..self.board.len() / 2 {
            let a = -self.board[i];
            let j = self.board.len() - 1 - i;
            self.board[i] = -self.board[j];
            self.board[j] = a;
        }
        

        swap(&mut self.active_bar, &mut self.inactive_bar);
        swap(&mut self.active_home, &mut self.inactive_home);
    }

    pub fn get_active_player(&self) -> Player {
        self.active_player
    }

    fn inactive_home_board(&self) -> &[i8; 6] {
        // SAFETY: board always has at least 24 elements, so 0..6 is valid
        unsafe { &*(self.board[0..6].as_ptr() as *const [i8; 6]) }
    }

    fn active_home_board(&self) -> &[i8; 6] {
        // SAFETY: board always has at least 24 elements, so 18..24 is valid
        unsafe { &*(self.board[18..24].as_ptr() as *const [i8; 6]) }
    }

    fn can_bear_off(&self) -> bool {
        let bar_is_empty = self.active_bar == 0;
        let checkers_in_home_board = self.active_home_board().iter().filter(|&&a| a > 0).sum::<i8>();
        
        bar_is_empty && checkers_in_home_board + self.active_home as i8 == 15
    }

    // Moves a checker from one position to another.
    // Fast but illegal moves can lead to undefined behavior.
    // Only use this function if you are sure the move is valid.
    pub fn make_half_move_unchecked(&mut self, half_move: &HalfMove) {
        match half_move.from.to_enum() {
            PositionEnum::Home => panic!("Cannot move from home position"),
            PositionEnum::Bar => self.active_bar -= 1,
            PositionEnum::Board(from) => self.board[from as usize] -= 1,
        }
        match half_move.to.to_enum() {
            PositionEnum::Home => self.active_home += 1,
            PositionEnum::Bar => panic!("Cannot move to bar position"),
            PositionEnum::Board(to) if self.board[to as usize] == -1 => {
                self.board[to as usize] = 1;
                self.inactive_bar += 1;
            },
            PositionEnum::Board(to) => self.board[to as usize] += 1,
        }
    }


    pub fn make_move_unchecked(&mut self, m: Move) {
        for half_move in m.half_moves.iter() {
            self.make_half_move_unchecked(half_move);
        }
        self.switch_player();
    }

    pub fn generage_moves(&self, dice: Dice) -> Vec<Move> {
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
                if dice.is_used() {
                    continue;
                }
                
                let half_moves = board.generate_half_moves(dice);
                
                for &(hv, remaining_dice) in half_moves.iter() {
                    let mut board = board.clone();
                    board.make_half_move_unchecked(&hv);
                    let mut mv = previous_moves;
                    mv.append(hv);
                    next_stack.push((remaining_dice, board, mv));
                }
            }
            if next_stack.is_empty() {
                break;
            }
            stack.clear();
            for e in next_stack.iter() {
                if !stack.iter().any(|(_, _, mv)| mv.unordered_equal(&e.2)) {
                    stack.push(*e);
                }
            }
            next_stack.clear();
        }
        results
    }

    pub fn generate_half_moves(&self, dice: Dice) -> TinyVector<(HalfMove, Dice), 30> {
        let available_dice = dice.get_unique_value();
        let mut half_moves = TinyVector::new();

        if self.active_bar == 0 {
            for &die in available_dice.iter() {
                for i in 0..(self.board.len() - die as usize) {
                    if self.board[i] > 0 && self.board[i + die as usize] >= -1 {
                        half_moves.push((
                            HalfMove {
                                from: Position::from_enum(PositionEnum::Board(i as u8)),
                                to: Position::from_enum(PositionEnum::Board((i + die as usize) as u8)),
                            },
                            dice.use_die(die),
                        ));
                    }
                }        
            }
            if self.can_bear_off() {
                for &die in available_dice.iter() {
                    let indx = 6 - die as usize;
                    if self.active_home_board()[indx] > 0 {
                        half_moves.push((
                            HalfMove {
                                from: Position::from_enum(PositionEnum::Board(24 - die)),
                                to: Position::from_enum(PositionEnum::Home),
                            },
                            dice.use_die(die),
                        ));
                    }
                }
                if half_moves.is_empty() {
                    for &die in available_dice.iter() {
                        let indx = 6 - die as usize;
                        for (i,v) in self.active_home_board()[indx..].iter().enumerate() {
                            if *v > 0 {
                                half_moves.push((
                                    HalfMove {
                                        from: Position::from_enum(PositionEnum::Board(18 + indx as u8 + i as u8)),
                                        to: Position::from_enum(PositionEnum::Home),
                                    },
                                    dice.use_die(die),
                                ));
                            }
                        }
                    }
                }
            }
        } else {
            for &die in available_dice.iter() {
                if self.inactive_home_board()[die as usize - 1] >= -1 {
                    half_moves.push((
                        HalfMove {
                            from: Position::from_enum(PositionEnum::Bar),
                            to: Position::from_enum(PositionEnum::Board(die - 1)),
                        },
                        dice.use_die(die),
                    ));
                }
            }
        }

        half_moves
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionEnum {
    Home,
    Bar,
    Board(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    position: NonZeroU8,
}

impl Position {
    pub fn from_enum(position: PositionEnum) -> Self {
        match position {
            PositionEnum::Board(n) => Position { position: NonZeroU8::new(n+1).unwrap() },
            PositionEnum::Bar => Position { position: NonZeroU8::new(254).unwrap() },
            PositionEnum::Home => Position { position: NonZeroU8::new(255).unwrap() },
        }
    }
    pub fn to_enum(&self) -> PositionEnum {
        match self.position.get() {
            255 => PositionEnum::Home,
            254 => PositionEnum::Bar,
            n => PositionEnum::Board(n - 1),
        }
    } 
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HalfMove {
    pub from: Position,
    pub to: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    half_moves: TinyVector<HalfMove, 4>,
}

impl Move {
    pub fn unordered_equal(&self, other: &Move) -> bool {
        let mut used: u8 = 0;
        for half_move in self.half_moves.iter() {
            match other.half_moves.iter().enumerate().position(|(i,&hm)| hm == *half_move && !used & (1 << i) != 0) {
                Some(index) => used |= 1 << index,
                None => return false,
            }       
        }
        true 
    }

    pub fn new() -> Self {
        Move { half_moves: TinyVector::new() }
    }

    pub fn append(&mut self, half_move: HalfMove) {
        self.half_moves.push(half_move);
    }

    pub fn len(&self) -> usize {
        self.half_moves.iter().count()
    }

    pub fn get_half_moves(&self) -> impl Iterator<Item = &HalfMove> {
        self.half_moves.iter()
    }

    pub fn remove_half_move(&mut self, half_move: &HalfMove) {
        self.half_moves.remove(half_move);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for half_move in self.half_moves.iter() {
            match half_move.from.to_enum() {
                PositionEnum::Home => result.push_str("H"),
                PositionEnum::Bar => result.push_str("B"),
                PositionEnum::Board(from) => result.push_str(&(from + 1).to_string()),
            }
            result.push_str(" -> ");
            match half_move.to.to_enum() {
                PositionEnum::Home => result.push_str("H"),
                PositionEnum::Bar => result.push_str("B"),
                PositionEnum::Board(to) => result.push_str(&(to + 1).to_string()),
            }
            result.push_str(", ");
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dice {
    Double{
        value: u8, 
        used: u8
    },
    Single{
        value_1: u8,
        value_2: u8,
        used: DiceUsage,
    }
}

impl Dice {
    pub const fn new(a: u8, b: u8) -> Dice {
        if a == b {
            Dice::Double { value: a, used: 0 }
        } else {
            Dice::Single { value_1: a, value_2: b, used: DiceUsage::BothAvailable }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Dice::Double { value, .. } => format!("{}/{}", value, value),
            Dice::Single { value_1, value_2, .. } => format!("{}/{}", value_1, value_2),
        }
    }

    pub fn roll() -> Dice {
        let a = random_range(1..=6);
        let b = random_range(1..=6);
        Dice::new(a, b)
    }

    fn is_used(&self) -> bool {
        match self {
            Dice::Double { used, .. } => *used >= 4,
            Dice::Single { used, .. } => *used == DiceUsage::BothUsed,
        }
    }

    fn get_unique_value(&self) -> TinyVector<u8, 2> {
        match self {
            &Dice::Double { value, used } => {
                if used < 4 {
                    TinyVector::from_raw([Some(value), None], 1)
                } else {
                    TinyVector::new()
                } 
            },
            &Dice::Single { value_1, value_2, used } => match used {
                DiceUsage::BothAvailable => {
                    TinyVector::from_raw([Some(value_1), Some(value_2)], 2)
                },
                DiceUsage::OnlyFirstAvailable => {
                    TinyVector::from_raw([Some(value_1), None], 1)
                },
                DiceUsage::OnlySecondAvailable => {
                    TinyVector::from_raw([Some(value_2), None], 1)
                },
                DiceUsage::BothUsed => TinyVector::new(),
            },
        }
    }

    fn use_die(&self, die: u8) -> Dice {
        let mut new_dice = *self;
        match &mut new_dice {
            Dice::Double { used, .. } => *used += 1,
            Dice::Single { value_1, value_2, used } => {
                if *value_1 == die {
                    *used = match *used {
                        DiceUsage::BothAvailable => DiceUsage::OnlySecondAvailable,
                        DiceUsage::OnlyFirstAvailable => DiceUsage::BothUsed,
                        _ => panic!("Cannot use die that is already used"),
                    };
                } else if *value_2 == die {
                    *used = match *used {
                        DiceUsage::BothAvailable => DiceUsage::OnlyFirstAvailable,
                        DiceUsage::OnlySecondAvailable => DiceUsage::BothUsed,
                        _ => panic!("Cannot use die that is already used"),
                    };
                }
            },
        }
        new_dice
    }

    pub const ALL: [Dice; 21] = [
        Dice::new(1, 1),
        Dice::new(1, 2),
        Dice::new(1, 3),
        Dice::new(1, 4),
        Dice::new(1, 5),
        Dice::new(1, 6),
        Dice::new(2, 2),
        Dice::new(2, 3),
        Dice::new(2, 4),
        Dice::new(2, 5),
        Dice::new(2, 6),
        Dice::new(3, 3),
        Dice::new(3, 4),
        Dice::new(3, 5),
        Dice::new(3, 6),
        Dice::new(4, 4),
        Dice::new(4, 5),
        Dice::new(4, 6),
        Dice::new(5, 5),
        Dice::new(5, 6),
        Dice::new(6, 6),
    ];

    pub const ALL_WITH_PROPABILITY: [(Dice, f32); 21] = [
        (Dice::new(1, 1), 0.0833),
        (Dice::new(1, 2), 0.1667),
        (Dice::new(1, 3), 0.1667),
        (Dice::new(1, 4), 0.1667),
        (Dice::new(1, 5), 0.1667),
        (Dice::new(1, 6), 0.1667),
        (Dice::new(2, 2), 0.0833),
        (Dice::new(2, 3), 0.1667),
        (Dice::new(2, 4), 0.1667),
        (Dice::new(2, 5), 0.1667),
        (Dice::new(2, 6), 0.1667),
        (Dice::new(3, 3), 0.0833),
        (Dice::new(3, 4), 0.1667),
        (Dice::new(3, 5), 0.1667),
        (Dice::new(3, 6), 0.1667),
        (Dice::new(4, 4), 0.0833),
        (Dice::new(4, 5), 0.1667),
        (Dice::new(4, 6), 0.1667),
        (Dice::new(5, 5), 0.0833),
        (Dice::new(5, 6), 0.1667),
        (Dice::new(6, 6), 0.0833),
    ];

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TinyVector<T, const N: usize> {
    data: [Option<T>; N],
    len: u8,
}

impl<T, const N: usize> TinyVector<T, N> 
where T: Clone + PartialEq 
{
    pub fn new() -> Self {
        TinyVector {
            data: [const { None }; N],
            len: 0,
        }
    }

    pub fn from_raw(array: [Option<T>; N], len: u8) -> Self {
        TinyVector {
            data: array,
            len 
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len < N as u8 {
            self.data[self.len as usize] = Some(value);
            self.len += 1;
        } else {
            panic!("TinyVector is full");
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().take_while(|&x| x.is_some()).flatten()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len as usize {
            self.data[index].as_ref()
        } else {
            None
        }
    }

    pub fn remove(&mut self, element: &T) {
        if let Some(pos) = self.data.iter().position(|x| x.as_ref() == Some(element)) {
            self.data[pos] = None;
            self.len -= 1;
            for i in pos..(self.len as usize) {
                self.data[i] = self.data[i + 1].take();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiceUsage {
    BothAvailable,
    OnlyFirstAvailable,
    OnlySecondAvailable,
    BothUsed,
}