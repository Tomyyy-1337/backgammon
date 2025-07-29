use std::{cell::{Cell, RefCell}, cmp::Reverse, f32::NEG_INFINITY, iter, rc::Rc};

use hashbrown::HashMap;

use nannou::{prelude::Pow, rand::{random, seq::SliceRandom, thread_rng}};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::game::{Board, Dice, GameOutcome, Move, Player};

pub fn find_best_move(board: &Board, dice: Dice, depth: u8) -> Move {
    let legal_moves = board.generate_moves(dice);
    
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
    
    let legal_moves = board.generate_moves(dice);
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
    let legal_moves = board.generate_moves(dice);

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

fn choose_random_move(board: &Board, dice: Dice) -> Move {
    let legal_moves = board.generate_moves(dice);
    
    if legal_moves.is_empty() {
        panic!("No legal moves available");
    }

    *legal_moves.choose(&mut thread_rng()).expect("No moves available")
}

fn find_highest_eval_move(board: &Board, dice: Dice) -> Move {
    let legal_moves = board.generate_moves(dice);
    
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

// use rand::prelude::*;
use std::f32::consts::SQRT_2;


const EXPLORATION_CONSTANT: f32 = SQRT_2;
const ROLLOUT_DEPTH: usize = 2;

// Node in the MCTS: either a player-decision node or a chance (dice-roll) node.
enum Node {
    Player(PlayerNode),
    Chance(ChanceNode),
}

struct PlayerNode {
    board: Board,
    dice: Dice,
    visits: u32,
    total_value: f32,
    untried_moves: Vec<Move>,
    children: Vec<(Move, Node)>,
}

struct ChanceNode {
    board: Board,
    visits: u32,
    total_value: f32,
    untried_rolls: Vec<Dice>,
    children: Vec<(Dice, Node)>,
}

impl PlayerNode {
    fn new(board: Board, dice: Dice) -> Self {
        let untried_moves = board.generate_moves(dice);
        PlayerNode {
            board,
            dice,
            visits: 0,
            total_value: 0.0,
            untried_moves,
            children: Vec::new(),
        }
    }

    fn traverse(&mut self, root_player: Player) -> f32 {
        if self.board.outcome() != GameOutcome::Ongoing {
            let val = self.board.eval();
            return if self.board.active_player() == root_player { val } else { -val };
        }

        if let Some(mov) = self.untried_moves.pop() {
            let mut next_board = self.board.clone();
            next_board.make_move_unchecked(mov);
            let mut child_node = Node::Chance(ChanceNode::new(next_board));
            let reward = match &mut child_node {
                Node::Chance(cn) => cn.simulate(root_player),
                _ => unreachable!(),
            };
            if let Node::Chance(cn) = child_node {
                self.children.push((mov, Node::Chance(cn)));
                let last = self.children.len() - 1;
                if let Node::Chance(cn2) = &mut self.children[last].1 {
                    cn2.visits = 1;
                    cn2.total_value = reward;
                }
            }
            self.visits += 1;
            self.total_value += reward;
            return reward;
        }

        let mut best_score = f32::NEG_INFINITY;
        let mut best_index = 0;
        for (i, (_, child)) in self.children.iter_mut().enumerate() {
            let (child_visits, child_value) = match child {
                Node::Player(pn) => (pn.visits as f32, pn.total_value),
                Node::Chance(cn) => (cn.visits as f32, cn.total_value),
            };
            let exploitation = child_value / child_visits;
            let exploration = EXPLORATION_CONSTANT * ((self.visits as f32).ln() / child_visits).sqrt();
            let score = exploitation + exploration;
            if score > best_score {
                best_score = score;
                best_index = i;
            }
        }

        let reward = match &mut self.children[best_index].1 {
            Node::Player(pn) => pn.traverse(root_player),
            Node::Chance(cn) => cn.traverse(root_player),
        };
        self.visits += 1;
        self.total_value += reward;
        reward
    }
}

impl ChanceNode {
    fn new(board: Board) -> Self {
        ChanceNode {
            board,
            visits: 0,
            total_value: 0.0,
            untried_rolls: Dice::ALL.to_vec(),
            children: Vec::new(),
        }
    }

    fn simulate(&mut self, root_player: Player) -> f32 {
        simulate_rollout(self.board, None, root_player)
    }

    fn traverse(&mut self, root_player: Player) -> f32 {
        if self.board.outcome() != GameOutcome::Ongoing {
            let val = self.board.eval();
            return if self.board.active_player() == root_player { val } else { -val };
        }

        if let Some(dice) = self.untried_rolls.pop() {
            let mut child_node = Node::Player(PlayerNode::new(self.board, dice));
            let reward = match &mut child_node {
                Node::Player(pn) => simulate_rollout(pn.board, Some(pn.dice), root_player),
                _ => unreachable!(),
            };
            if let Node::Player(pn) = child_node {
                self.children.push((dice, Node::Player(pn)));
                let last = self.children.len() - 1;
                if let Node::Player(pn2) = &mut self.children[last].1 {
                    pn2.visits = 1;
                    pn2.total_value = reward;
                }
            }
            self.visits += 1;
            self.total_value += reward;
            return reward;
        }

        let mut rng = thread_rng();
        let r: f32 = nannou::rand::random();
        let mut cum = 0.0;
        let mut chosen_index = 0;
        for (i, (dice, _)) in self.children.iter().enumerate() {
            cum += dice.probability();
            if r < cum {
                chosen_index = i;
                break;
            }
        }

        let reward = match &mut self.children[chosen_index].1 {
            Node::Player(pn) => pn.traverse(root_player),
            Node::Chance(cn) => cn.traverse(root_player),
        };
        self.visits += 1;
        self.total_value += reward;
        reward
    }
}

fn simulate_move_eval(board: Board, mv: Move) -> f32 {
    let mut next = board.clone();
    next.make_move_unchecked(mv);
    next.eval()
}

fn simulate_rollout(mut board: Board, mut opt_dice: Option<Dice>, root_player: Player) -> f32 {
    let mut rng = thread_rng();
    for _ in 0..ROLLOUT_DEPTH {
        if board.outcome() != GameOutcome::Ongoing {
            break;
        }

        let dice = opt_dice.take().unwrap_or_else(|| {
            let r: f32 = nannou::rand::random();
            let mut cum = 0.0;
            for d in Dice::ALL {
                cum += d.probability();
                if r < cum {
                    return d;
                }
            }
            Dice::ALL[Dice::ALL.len() - 1]
        });

        let moves = board.generate_moves(dice);
        if !moves.is_empty() {
            let len = moves.len();
    
            let indx = (random::<f32>().pow(16) * (len as f32 - 1.0)) as usize; 
            // println!("Choosing move at index: {}\\{}", indx, len);
        
            let mut evals = moves.into_iter()
                .map(|m| {
                    let mut new_board = board.clone();
                    new_board.make_move_unchecked(m);
                    (m, new_board.eval())
                })
                .collect::<Vec<_>>();
        
            evals.sort_unstable_by(|(_, eval1), (_, eval2)| eval1.partial_cmp(&eval2).unwrap());
            let best = evals
                .into_iter()
                .nth(indx)
                .map(|(m, _)| m)
                .expect("No moves available");
            board.make_move_unchecked(best);
        }
    }

    let val = board.eval();
    if board.active_player() == root_player { val } else { -val }
}

pub fn mcts_search(root_board: Board, dice: Dice, iterations: u32) -> Move {
    let root_player = root_board.active_player();
    let mut root_node = Node::Player(PlayerNode::new(root_board, dice));

    for _ in 0..iterations {
        let _ = match &mut root_node {
            Node::Player(pn) => pn.traverse(root_player),
            Node::Chance(cn) => cn.traverse(root_player),
        };
    }

    if let Node::Player(pn) = root_node {
        let mut best_move = pn.untried_moves.first().cloned()
            .unwrap_or_else(|| pn.children[0].0);
        let mut best_visits = 0;
        for (mv, child) in pn.children {
            let visits = match child {
                Node::Player(pn) => pn.visits,
                Node::Chance(cn) => cn.visits,
            };
            if visits > best_visits {
                best_visits = visits;
                best_move = mv;
            }
        }
        best_move
    } else {
        panic!("Root node must be a PlayerNode");
    }
}
