use std::{cell::{Cell, RefCell}, cmp::Reverse, iter, rc::Rc};

use hashbrown::HashMap;

use nannou::{prelude::Pow, rand::{random, seq::SliceRandom, thread_rng}};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::game::{Board, Dice, GameOutcome, Move, Player};

pub fn find_best_move(board: &Board, dice: Dice, depth: u8) -> Move {
    let legal_moves = board.generage_moves(dice);
    
    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    let evals = legal_moves.into_par_iter()
        .map(|m| {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(m);
            let mut seen = HashMap::new();
            let eval = -alpha_beta(&new_board, depth, f32::NEG_INFINITY, f32::INFINITY, dice, &mut seen);
            (m, eval)
        })
        .collect::<Vec<_>>();
        
        let best_move = evals.into_iter()
            .max_by(|(_, eval1), (_, eval2)| eval1.partial_cmp(eval2).unwrap())
            .expect("No moves available");
        best_move.0
}

pub fn search_eval(board: &Board, depth: u8) -> f32 {
    let mut seen = HashMap::new();
    average_eval(board, f32::NEG_INFINITY, f32::INFINITY, depth, &mut seen)
}

fn average_eval(board: &Board, alpha: f32, beta: f32, depth: u8, seen: &mut HashMap<(Board,Dice), f32>) -> f32 {
    let mut sum = 0.0;
    for (dice, propability) in Dice::ALL_WITH_PROPABILITY {
        let eval = alpha_beta(board, depth, alpha, beta, dice, seen);
        sum += eval * propability;
    }
    sum 
}

fn alpha_beta(board: &Board, depth: u8, mut alpha: f32, beta: f32, dice: Dice, seen: &mut HashMap<(Board,Dice), f32>) -> f32 {
    if depth == 0 {
        return board.eval();
    }

    if let Some(&cached_eval) = seen.get(&(*board, dice)) {
        return cached_eval;
    }
    
    let legal_moves = board.generage_moves(dice);
    if legal_moves.is_empty() {
        return board.eval();
    }

    // legal_moves.sort_unstable_by_key(
    //     |m| std::cmp::Reverse(board.captured_value(&m))
    // );

    let mut best_eval = f32::NEG_INFINITY;

    for m in legal_moves {
        let mut new_board = board.clone();
        new_board.make_move_unchecked(m);
        let eval = -average_eval(&new_board, -beta, -alpha, depth - 1, seen);
        
        best_eval = best_eval.max(eval);
        alpha = alpha.max(best_eval);

        if beta <= alpha {
            break;
        }
    }

    seen.insert((*board, dice), best_eval);
    best_eval
}

pub fn monte_carlo_search(board: &Board, dice: Dice, simulations: usize, depth: usize) -> Move {
    let legal_moves = board.generage_moves(dice);

    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    legal_moves.into_par_iter() 
        .map(|m| {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(m.clone());
            let mut score = 0.0;
            for _ in 0..simulations {
                match board.get_active_player() {
                    Player::White => score += simulate_random_game(&new_board, depth),
                    Player::Black => score -= simulate_random_game(&new_board, depth),
                }
            }
            (m, score / simulations as f32)
        })
        .max_by(|(_, score1), (_, score2)| {
            score1.partial_cmp(score2).unwrap()
        })
        .map(|(m, _)| m)
        .expect("No moves available")
}

pub fn monte_carlo_search_2(board: &Board, dice: Dice, simulations: usize, depth: usize) -> Move {
    let legal_moves = board.generage_moves(dice);

    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    legal_moves.into_par_iter() 
        .map(|m| {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(m.clone());
            let mut score = 0.0;
            for _ in 0..simulations {
                match board.get_active_player() {
                    Player::White => score += simulate_random_game_2(&new_board, depth),
                    Player::Black => score -= simulate_random_game_2(&new_board, depth),
                }
            }
            (m, score / simulations as f32)
        })
        .max_by(|(_, score1), (_, score2)| {
            score1.partial_cmp(score2).unwrap()
        })
        .map(|(m, _)| m)
        .expect("No moves available")
}

fn simulate_random_game(board: &Board, depth: usize) -> f32 {
    let mut current_board = board.clone();

    for _ in 0..depth { 
        let dice = Dice::roll();

        if GameOutcome::Ongoing != current_board.outcome() {
            break; 
        } 
        let m = find_highest_eval_move(&current_board, dice);
        
        // let m = find_highest_eval_move(&current_board, dice);
        
        current_board.make_move_unchecked(m);
    }
    
    current_board.eval_absolute()
}

fn simulate_random_game_2(board: &Board, depth: usize) -> f32 {
    let mut current_board = board.clone();
    
    for _ in 0..depth { 
        let dice = Dice::roll();
        
        if GameOutcome::Ongoing != current_board.outcome() {
            break;
        } 
        let m = find_highest_eval_move_2(&current_board, dice);

        // let m = find_highest_eval_move(&current_board, dice);

        current_board.make_move_unchecked(m);
    }

    current_board.eval_absolute()
}

fn choose_random_move(board: &Board, dice: Dice) -> Move {
    let legal_moves = board.generage_moves(dice);
    
    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    *legal_moves.choose(&mut thread_rng()).expect("No moves available")
}

fn find_highest_eval_move(board: &Board, dice: Dice) -> Move {
    let legal_moves = board.generage_moves(dice);
    
    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    // legal_moves.shuffle(&mut thread_rng());
    let len = legal_moves.len();
    
    let indx = (random::<f32>().pow(16) * (len as f32 - 1.0)) as usize; 
    // println!("Choosing move at index: {}\\{}", indx, len);

    let mut evals = legal_moves.into_iter()
        .map(|m| {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(m);
            (m, new_board.eval())
        })
        .collect::<Vec<_>>();

    evals.sort_unstable_by(|(_, eval1), (_, eval2)| eval1.partial_cmp(&eval2).unwrap());
    evals
        .into_iter()
        .nth(indx)
        .map(|(m, _)| m)
        .expect("No moves available")
}

fn find_highest_eval_move_2(board: &Board, dice: Dice) -> Move {
    let legal_moves = board.generage_moves(dice);
    
    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    // legal_moves.shuffle(&mut thread_rng());
    let len = legal_moves.len();
    
    let indx = (random::<f32>().pow(16) * (len as f32 - 1.0)) as usize; 
    // println!("Choosing move at index: {}\\{}", indx, len);

    let mut evals = legal_moves.into_iter()
        .map(|m| {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(m);
            (m, new_board.simple_eval())
        })
        .collect::<Vec<_>>();

    evals.sort_unstable_by(|(_, eval1), (_, eval2)| eval1.partial_cmp(&eval2).unwrap());
    evals
        .into_iter()
        .nth(indx)
        .map(|(m, _)| m)
        .expect("No moves available")
}
