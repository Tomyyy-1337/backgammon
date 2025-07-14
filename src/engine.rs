use hashbrown::HashMap;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::game::{Board, Dice, Move};

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
    
    let mut legal_moves = board.generage_moves(dice);
    if legal_moves.is_empty() {
        return board.eval();
    }

    legal_moves.sort_unstable_by_key(
        |m| std::cmp::Reverse(board.captured_value(&m))
    );

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
