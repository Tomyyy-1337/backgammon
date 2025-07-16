#![windows_subsystem = "windows"]

use std::usize;

use backgammon::{engine::{find_best_move, monte_carlo_search}, game::{self, Board, Dice, GameOutcome, HalfMoveEnum, Move, Player, Position, PositionEnum}};
use nannou::{color::WHITE, geom::Rect, wgpu::Backends};
use rand::{rng, seq::IteratorRandom};

fn main() {
    // run_games();
    // benchmark();

    // let depth = 2;
    // println!("Starting performance test with depth {}", depth);
    // let board = Board::new();
    // let start = std::time::Instant::now();
    // let count = performance_test(&board, depth);
    // let duration = start.elapsed();
    // println!("Performance test completed in {:?} with {} moves evaluated", duration, count);

    nannou::app(model).backends(Backends::DX12).update(update).run();
}

struct Model {
    board: Board,
    games_played: u32,
    wins: (u32, u32),
    gammons: (u32, u32),
    backgammons: (u32, u32),
    current_dice: Option<Dice>,
    state: State,
    pending_move_part: Option<game::PositionEnum>,
    mous_is_down: bool,
    available_moves: Vec<game::Move>,
    engine_thread: Option<std::thread::JoinHandle<game::Move>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    RollDice,
    ChooseMove,
    ShowMove{
        mv: Move,
        indx: usize,
        timer: u8,
    },
    UserMove,
}

fn update(app: &nannou::App, model: &mut Model, update: nannou::event::Update) {
    let frames_per_second = (1.0 / update.since_last.as_secs_f32()) as usize;

    match model.state {
        State::RollDice => {
            model.current_dice = Some(Dice::roll());
            model.state = State::ChooseMove;
        }
        State::ShowMove{mv, indx, timer} if indx == mv.len() && timer == 0 => {
            model.board.switch_player();
            model.state = State::UserMove;
            model.current_dice = Some(Dice::roll());
            model.available_moves = model.board.generage_moves(model.current_dice.unwrap());
            outcome(model);
        }
        State::ShowMove{mv, indx, timer} if timer == 0 => {
            model.board.make_half_move_unchecked(mv.get_half_moves().nth(indx).unwrap());
            model.state = State::ShowMove{mv, timer: (frames_per_second / 2) as u8, indx: indx + 1};
        }
        State::ShowMove{mv, indx, timer} => {
            model.state = State::ShowMove{mv, indx, timer: timer - 1};
        }
        State::ChooseMove => {
            match &model.engine_thread {
                None => {
                    let board = model.board.clone();
                    let dice = model.current_dice.unwrap();
                    model.engine_thread = Some(std::thread::spawn(move || {
                        // let best_move = match board.get_active_player() {
                        //     Player::White => find_best_move(&board, dice, 2),
                        //     Player::Black => choose_random_move(&board, dice),
                        // };
                        // best_move
                        monte_carlo_search(&board, dice, 400, 20)
                    }));
                },
                Some(thread) => {
                    if !thread.is_finished() {
                        return;
                    }
                    let thread = model.engine_thread.take().unwrap();
                    let best_move = thread.join().expect("Failed to join engine thread");

                    model.state = State::ShowMove{mv: best_move, indx: 0, timer: (frames_per_second / 2) as u8};
                }
            }
        }
        State::UserMove => {
            if model.available_moves[0].len() == 0 {
                model.state = State::RollDice;
                model.board.switch_player();
            }

            if model.available_moves.is_empty() {
                model.state = State::RollDice;
                model.board.make_move_unchecked(Move::new());
                return;
            }

            if let Some(current_mouse_pos) = mouse_pos_to_board_pos(app) {
                if !app.mouse.buttons.left().is_down() {
                    model.mous_is_down = false;
                    return;
                } else if model.mous_is_down {
                    return;
                }
                model.mous_is_down = true;
                if let Some(pending) = model.pending_move_part {
                    if model.available_moves.iter().flat_map(|m|m.get_half_moves()).any(|m| m.to == current_mouse_pos && m.from == Position::from_enum(pending)) {
                        let halfmove = HalfMoveEnum { from: Position::from_enum(pending), to: current_mouse_pos };
                        model.pending_move_part = None;
                        model.board.make_half_move_unchecked(&halfmove);
                        model.available_moves = model.available_moves.drain(..)
                            .filter(|m| m.get_half_moves().any(|hm| *hm == halfmove))
                            .collect();
                        for mv in model.available_moves.iter_mut() {
                            mv.remove_half_move(&halfmove);  
                        }
                    } else {
                        model.pending_move_part = None;
                    }
                } else {
                    if model.available_moves.iter().flat_map(|m|m.get_half_moves()).any(|m| m.from == current_mouse_pos) {
                        model.pending_move_part = Some(current_mouse_pos.to_enum());
                    }
                }       
            }
            match model.board.outcome() {
                GameOutcome::Ongoing => (),
                GameOutcome::Win(player) => {
                    match player {
                        Player::White => model.wins.0 += 1,
                        Player::Black => model.wins.1 += 1,
                    }
                    model.games_played += 1;
                    model.board = Board::new();
                }
                GameOutcome::Gammon(player) => {
                    model.board = Board::new();
                    match player {
                        Player::White => model.gammons.0 += 1,
                        Player::Black => model.gammons.1 += 1,  
                    }
                    model.games_played += 1;
                    model.board = Board::new();
                }
                GameOutcome::Backgammon(player) => {
                    match player {
                        Player::White => model.backgammons.0 += 1,
                        Player::Black => model.backgammons.1 += 1,  
                    }
                    model.games_played += 1;
                    model.board = Board::new();
                }
            }
            outcome(model);
        }
    }


}

fn outcome(model: &mut Model) {
    match model.board.outcome() {
        GameOutcome::Ongoing => (),
        GameOutcome::Win(player) => {
            match player {
                Player::White => model.wins.0 += 1,
                Player::Black => model.wins.1 += 1,
            }
            model.games_played += 1;
            model.board = Board::new();
            model.state = State::RollDice;
        }
        GameOutcome::Gammon(player) => {
            model.board = Board::new();
            match player {
                Player::White => model.gammons.0 += 1,
                Player::Black => model.gammons.1 += 1,  
            }
            model.games_played += 1;
            model.board = Board::new();
            model.state = State::RollDice;
        }
        GameOutcome::Backgammon(player) => {
            match player {
                Player::White => model.backgammons.0 += 1,
                Player::Black => model.backgammons.1 += 1,  
            }
            model.games_played += 1;
            model.board = Board::new();
            model.state = State::RollDice;
        }
    }
}

fn model(app: &nannou::App) -> Model {
    app.new_window()
        .view(view)
        .build()
        .unwrap();
    
    Model {
        board: Board::new(),
        games_played: 0,
        wins: (0, 0),
        gammons: (0, 0),
        backgammons: (0, 0),
        current_dice: None, 
        state: State::RollDice,
        pending_move_part: None,
        mous_is_down: false,
        available_moves: Vec::new(),
        engine_thread: None,
    }
}

fn mouse_pos_to_board_pos(app: &nannou::App) -> Option<Position> {
    let mouse = app.mouse.position();
    let window_rect = app.window_rect();
    let (width, height) = (window_rect.w(), window_rect.h());
    let (center_x, center_y) = (window_rect.x(), window_rect.y());

    let stats_rect_width = 320.0;
    let board_rect = Rect::from_w_h(width - stats_rect_width, height).shift_x(center_x - stats_rect_width / 2.0).shift_y(center_y);

    let x = mouse.x - board_rect.left(); 
    let y = mouse.y - center_y;

    let tile_width = board_rect.w() / 13.0;
    let tile_index = x as usize / tile_width as usize;

    if tile_index >= 13 {
        return Some(Position::from_enum(PositionEnum::Home));
    }

    if tile_index == 6 {
        return Some(Position::from_enum(PositionEnum::Bar));
    } else if y > 0.0 {
        let indx = if tile_index < 6 { tile_index + 12 } else { tile_index + 11 };
        return Some(Position::from_enum(PositionEnum::Board(indx as u8)));
    } else {
        let indx = if tile_index < 6 { 11 - tile_index } else { 12 - tile_index };
        return Some(Position::from_enum(PositionEnum::Board(indx as u8)));
    } 
}

fn view(app: &nannou::App, model: &Model, frame: nannou::frame::Frame) {
    let draw = app.draw();
    draw.background().color(nannou::color::BLACK);

    let window_rect = app.window_rect();
    let (width, height) = (window_rect.w(), window_rect.h());
    let (center_x, center_y) = (window_rect.x(), window_rect.y());

    let stats_rect_width = 320.0;
    let stats_rect = Rect::from_w_h(stats_rect_width, height).shift_x(width / 2.0 - stats_rect_width / 2.0).shift_y(center_y);
    let board_rect = Rect::from_w_h(width - stats_rect_width, height).shift_x(center_x - stats_rect_width / 2.0).shift_y(center_y);

    draw.rect()
        .x_y(board_rect.x(), board_rect.y())
        .w_h(board_rect.w(), board_rect.h())
        .color(nannou::color::BLUE);

    draw.rect()
        .x_y(stats_rect.x(), stats_rect.y())
        .w_h(stats_rect.w(), stats_rect.h())
        .color(nannou::color::BLACK);

    // Draw board
    let tile_width = board_rect.w() / 13.0;
    let tile_height = 2.0 * board_rect.h() / 5.0;

    for i in 0..13 {
        if i == 6 {
            draw.rect()
                .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, 0.0)
                .w_h(tile_width, board_rect.h())
                .color(nannou::color::BLACK);

            if let State::UserMove = model.state && model.pending_move_part.is_none() {
                if model.board.bar(Player::Black) > 0 {
                    draw.rect()
                        .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.y() - board_rect.h() / 4.0)
                        .w_h(tile_width, board_rect.h() / 2.0)
                        .no_fill()
                        .stroke_weight(2.0)
                        .stroke(WHITE);
                }
            }
            
            let white_bar = model.board.bar(Player::White);
            let black_bar = model.board.bar(Player::Black);

            if white_bar > 0 {
                draw.ellipse()
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.top() - board_rect.h() / 3.9)
                    .w_h(tile_width * 0.7, tile_width * 0.7)
                    .color(nannou::color::WHITE);
                draw.text(&white_bar.to_string())
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.top() - board_rect.h() / 4.0)
                    .font_size(tile_width as u32 / 2)
                    .color(nannou::color::BLACK);
            }

            if black_bar > 0 {
                draw.ellipse()
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.bottom() + board_rect.h() / 4.1)
                    .w_h(tile_width * 0.7, tile_width * 0.7)
                    .color(nannou::color::RED);
                draw.text(&black_bar.to_string())
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.bottom() + board_rect.h() / 4.0)
                    .font_size(tile_width as u32 / 2)
                    .color(nannou::color::WHITE);
            }

            continue;
        }
        let x = board_rect.left() + i as f32 * tile_width;
        let y = board_rect.top();
        let color_indx = if i < 6 { i } else { i + 1 };
        let color = if color_indx % 2 == 0 { nannou::color::DARKBLUE } else { nannou::color::BLACK };
        let indx = if i < 6 { 11-i } else { 12-i };
        let checkers = model.board.checkers_on_position(indx);
        draw.polygon()
            .points([
                (x, y),
                (x + tile_width, y),
                (x + tile_width / 2.0, y - tile_height),
            ])
            .color(color);

        if let State::UserMove = model.state {
            match model.pending_move_part {
                Some(pending) => {
                    if model.available_moves.iter().flat_map(|m| m.get_half_moves()).any(|hm| hm.from.to_enum() == pending && hm.to.to_enum() == PositionEnum::Board(23 - indx as u8)) {
                        draw.polyline()
                            .points_closed([
                                (x, y),
                                (x + tile_width, y),
                                (x + tile_width / 2.0, y - tile_height),
                            ])
                            .color(nannou::color::WHITE);
                    }
                }
                None if checkers < 0 => {
                    if model.available_moves.iter().flat_map(|m| m.get_half_moves()).any(|hm| hm.from.to_enum() == PositionEnum::Board(23 - indx as u8))  {
                        draw.polyline()
                            .points_closed([
                                (x, y),
                                (x + tile_width, y),
                                (x + tile_width / 2.0, y - tile_height),
                            ])
                            .color(nannou::color::WHITE);
                    }
                }
                _ => (),
            }
        } 
        
        if checkers != 0 {
            let color = if checkers > 0 { nannou::color::WHITE } else { nannou::color::RED };
            for i in 0..checkers.abs().min(5) {
                let y =  y - tile_width * 0.5 * (i as f32 + 0.5) - 5.0;
                draw.ellipse()
                    .x_y(x + tile_width / 2.0, y)
                    .w_h(tile_width * 0.5, tile_width * 0.5)
                    .color(color);
            }

            if checkers.abs() > 5 {
                let y = y - tile_width * 0.20 - 5.0;
                draw.text(&format!("+{}", &(checkers.abs()-5).to_string()))
                    .x_y(x + tile_width / 2.0, y)
                    .font_size(tile_width as u32 / 3)
                    .color(if checkers > 0 { nannou::color::BLACK } else { nannou::color::WHITE });
            }
        }

        let y = board_rect.bottom();
        let color = if color_indx % 2 == 0 { nannou::color::BLACK } else { nannou::color::DARKBLUE };
        draw.polygon()
            .points([
                (x, y),
                (x + tile_width, y),
                (x + tile_width / 2.0, y + tile_height),
            ])
            .color(color);

        let indx = if i < 6 { i + 12 } else { i + 11 };
        let checkers = model.board.checkers_on_position(indx);

        if let State::UserMove = model.state {
            match model.pending_move_part {
                Some(pending) => {
                    if model.available_moves.iter().flat_map(|m| m.get_half_moves()).any(|hm| hm.from.to_enum() == pending && hm.to.to_enum() == PositionEnum::Board(23 - indx as u8)) {
                        draw.polyline()
                            .points_closed([
                                (x, y),
                                (x + tile_width, y),
                                (x + tile_width / 2.0, y + tile_height),
                            ])
                            .color(nannou::color::WHITE);
                    }
                }
                None if checkers < 0 => {
                    if model.available_moves.iter().flat_map(|m| m.get_half_moves()).any(|hm| hm.from.to_enum() == PositionEnum::Board(23 - indx as u8))  {
                        draw.polyline()
                            .points_closed([
                                (x, y),
                                (x + tile_width, y),
                                (x + tile_width / 2.0, y + tile_height),
                            ])
                            .color(nannou::color::WHITE);
                    }
                }
                _ => (),
            }
        } 
        
        if checkers != 0 {
            let color = if checkers > 0 { nannou::color::WHITE } else { nannou::color::RED };
            for i in 0..checkers.abs().min(5) {
                let y = y + tile_width * 0.5 * (i as f32 + 0.5) + 5.0;
                draw.ellipse()
                .x_y(x + tile_width / 2.0, y)
                .w_h(tile_width * 0.5, tile_width * 0.5)
                .color(color);
        }
        
        if checkers.abs() > 5 {
            let y = y + tile_width * 0.28 + 5.0;
            draw.text(&format!("+{}", &(checkers.abs()-5).to_string()))
                .x_y(x + tile_width / 2.0, y)
                .font_size(tile_width as u32 / 3)
                .color(if checkers > 0 { nannou::color::BLACK } else { nannou::color::WHITE });
            }
        }
    }

    // Draw dice 
    if let Some(dice) = model.current_dice {
        let inverted = match model.state {
            State::RollDice => true,
            _ => false,
        };
        let player = match model.board.active_player() {
            Player::White => if inverted { Player::Black } else { Player::White },
            Player::Black => if inverted { Player::White } else { Player::Black },
        };
        match player {
            Player::White => {
                draw.text(&dice.to_string())
                    .x_y(-board_rect.w() / 4.0 + board_rect.x(), board_rect.h() / 40.0)
                    .font_size(board_rect.w() as u32 / 10)
                    .w(board_rect.w() / 2.0)
                    .color(nannou::color::WHITE);
            }
            Player::Black => {
                draw.text(&dice.to_string())
                    .x_y(board_rect.w() / 4.0 + board_rect.x(), board_rect.h() / 40.0)
                    .font_size(board_rect.w() as u32 / 10)
                    .w(board_rect.w() / 2.0)
                    .color(nannou::color::WHITE);
            }
        }
    }

    // Draw stats
    let mut y = stats_rect.top() - 50.0;
    let x = stats_rect.x();
    draw.text("Stats")
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(30)
        .color(nannou::color::WHITE);

    y -= 30.0;

    draw.text(&format!("Evaluation: {}", model.board.eval_absolute()))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    y -= 30.0;

    let total_wins_white = model.wins.0 + model.gammons.0 + model.backgammons.0;
    let total_wins_black = model.wins.1 + model.gammons.1 + model.backgammons.1;
    let winrate_white = if model.games_played > 0 {
        total_wins_white as f32 / model.games_played as f32 * 100.0
    } else {
        0.0
    };

    let winrate_black = if model.games_played > 0 {
        total_wins_black as f32 / model.games_played as f32 * 100.0
    } else {
        0.0
    };

    draw.text(&format!("Games Played: {}", model.games_played))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    y -= 30.0;

    draw.text(&format!("White Wins: {}, Winrate: {:.2}%", total_wins_white, winrate_white))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    y -= 30.0;

    draw.text(&format!("Black Wins: {}, Winrate: {:.2}%", total_wins_black, winrate_black))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    y -= 30.0;

    draw.text(&format!("White Gammons: {}, Backgammons: {}", model.gammons.0, model.backgammons.0))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    y -= 30.0;

    draw.text(&format!("Black Gammons: {}, Backgammons: {}", model.gammons.1, model.backgammons.1))
        .x_y(x, y)
        .w(stats_rect_width - 20.0)
        .font_size(16)
        .color(nannou::color::WHITE);

    // Draw eval_bar
    let eval_bar_width = 12.0;
    let x = stats_rect.left();
    let eval = -model.board.eval_absolute() * 3.0;

    draw.polygon()
        .points([
            (x, stats_rect.top()),
            (x + eval_bar_width, stats_rect.top()),
            (x + eval_bar_width, eval),
            (x, eval),
        ])
        .color(WHITE);



    draw.to_frame(app, &frame).unwrap();
}


fn run_games() {
    let mut games = 0;
    let mut white_wins = 0;
    let mut white_gammon = 0;
    let mut white_backgammon = 0;
    let mut black_wins = 0;
    let mut black_gammon = 0;
    let mut black_backgammon = 0;
    loop {
        let mut board = Board::new();
        games += 1;
        loop {
            print!("=========================================================\n");
            println!("{}", board.to_fancy_string());
            print!("Curent Evaluation: {}\n", board.eval_absolute());
            println!("White Home: {}, White Bar: {}, Black Home: {}, Black Bar: {}", board.home(Player::White), board.bar(Player::White), board.home(Player::Black), board.bar(Player::Black));              
            match board.outcome() {
                GameOutcome::Ongoing => {
                    let dice = Dice::roll();
                    
                    println!("{:?} rolled {}", board.get_active_player(), dice.to_string());

                    let start = std::time::Instant::now();
                    let mv = match board.get_active_player() {
                        Player::White => find_best_move(&board, dice, 1),
                        Player::Black => monte_carlo_search(&board, dice, 400, 20),
                    };
                    let duration = start.elapsed();
                    println!("{:?} moved {}", board.get_active_player(), mv.to_string());   
                    println!("Move evaluation took: {:?}", duration);
                    
                    board.make_move_unchecked(mv);
                }
                GameOutcome::Win(player) => {
                    println!("Player {:?} wins!", player);
                    match player {
                        Player::White => white_wins += 1,
                        Player::Black => black_wins += 1,
                    }
                    break;
                }
                GameOutcome::Gammon(player) => {
                    println!("Player {:?} wins with a gammon!", player);
                    match player {
                        Player::White => white_gammon += 1,
                        Player::Black => black_gammon += 1,
                    }
                    break;
                }
                GameOutcome::Backgammon(player) => {
                    println!("Player {:?} wins with a backgammon!", player);
                    match player {
                        Player::White => white_backgammon += 1,
                        Player::Black => black_backgammon += 1,
                    }
                    break;
                }
            }
        }
        let white_win_rate = (white_wins + white_gammon + white_backgammon) as f32 / games as f32 * 100.0;
        let black_win_rate = (black_wins + black_gammon + black_backgammon) as f32 / games as f32 * 100.0;
        let white_gammon_rate = white_gammon as f32 / games as f32 * 100.0;
        let white_backgammon_rate = white_backgammon as f32 / games as f32 * 100.0;
        let black_gammon_rate = black_gammon as f32 / games as f32 * 100.0;
        let black_backgammon_rate = black_backgammon as f32 / games as f32 * 100.0;
        println!("============================================");
        println!("Games: {}, White Wins: {}, Black Wins: {}", games, white_wins + white_gammon + white_backgammon, black_wins + black_gammon + black_backgammon);
        println!("White Win Rate: {:.2}%, Black Win Rate: {:.2}%", white_win_rate, black_win_rate);
        println!("White Gammon Rate: {:.2}%, White Backgammon Rate: {:.2}%", white_gammon_rate, white_backgammon_rate);
        println!("Black Gammon Rate: {:.2}%, Black Backgammon Rate: {:.2}%", black_gammon_rate, black_backgammon_rate);
        println!("============================================");

        let outcome = format!(
            "Games: {}, White Wins: {}, Black Wins: {},\nWhite Win Rate: {:.2}%, Black Win Rate: {:.2}%,\nWhite Gammon Rate: {:.2}%, Black Gammon Rate: {:.2}%,\nWhite Backgammon Rate: {:.2}%, Black Backgammon Rate: {:.2}%",
            games, white_wins + white_gammon + white_backgammon, black_wins + black_gammon + black_backgammon,
            white_win_rate, black_win_rate,
            white_gammon_rate, black_gammon_rate,
            white_backgammon_rate, black_backgammon_rate
        );

        std::fs::write("outcomes", outcome).expect("Unable to write file");
            
    }
}

fn choose_random_move(board: &Board, dice: Dice) -> game::Move {
    let moves = board.generage_moves(dice);
    if moves.is_empty() {
        panic!("No valid moves available");
    }
    moves.into_iter().choose(&mut rng()).expect("Failed to choose a random move")
}

fn benchmark() {
    let board = Board::bench();
    println!("Evaluation: {}", board.eval_absolute());
    let start = std::time::Instant::now();
    let depth = 3;
    
    let m = find_best_move(&board, Dice::new(1, 4), depth);
    let available_moves = board.generage_moves(Dice::new(1, 4));

    let board = Board::new();
    for dice in Dice::ALL {
        let legal_moves = board.generage_moves(dice);
        assert!(!legal_moves.is_empty(), "No legal moves available for dice: {:?}", dice);
    }

    let duration = start.elapsed();
    println!("Best move: {}", m.to_string());
    println!("Best moves found in {:?} for depth {}", duration, depth);
}

fn performance_test(board: &Board, depth: u32) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut sum = 0;
    for dice in Dice::ALL {
        let legal_moves = board.generage_moves(dice);
        for mv in legal_moves {
            let mut new_board = board.clone();
            new_board.make_move_unchecked(mv);
            sum += performance_test(&new_board, depth - 1);
        }
    }
    sum
}