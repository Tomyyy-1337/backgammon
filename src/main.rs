use std::thread::sleep;

use backgammon::{engine::{find_best_move}, game::{self, Board, Dice, GameOutcome, Player}};
use nannou::{color::{self, BLACK, WHITE}, geom::Rect};
use rand::{rng, seq::IteratorRandom};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    // run_games();
    // benchmark();

    nannou::app(model).update(update).run();

    
}

struct Model {
    board: Board,
    games_played: u32,
    wins: (u32, u32),
    gammons: (u32, u32),
    backgammons: (u32, u32),
    current_dice: Option<Dice>,
    state: State,
}

enum State {
    RollDice,
    ShowStatus(u8),
    ChooseMove,
}

fn update(_app: &nannou::App, model: &mut Model, _update: nannou::event::Update) {
    match model.state {
        State::RollDice => {
            model.current_dice = Some(Dice::roll());
            model.state = State::ShowStatus(0);
            return;
        }
        State::ShowStatus(n) if n == 30 => {
            model.state = State::ChooseMove;
            return;
        }
        State::ShowStatus(n) => {
            model.state = State::ShowStatus(n + 1);
            return;
        }
        State::ChooseMove => {
            model.state = State::RollDice;
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
    
    let best_move = match model.board.get_active_player() {
        Player::White => find_best_move(&model.board, model.current_dice.unwrap(), 2),
        Player::Black => choose_random_move(&model.board, model.current_dice.unwrap()),
    };

    model.current_dice = None;
    
    model.board.make_move_unchecked(best_move);
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
            
            let white_bar = model.board.bar(Player::White);
            let black_bar = model.board.bar(Player::Black);

            if white_bar > 0 {
                draw.ellipse()
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.top() - tile_height / 1.95)
                    .w_h(tile_width * 0.7, tile_width * 0.7)
                    .color(nannou::color::WHITE);
                draw.text(&white_bar.to_string())
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.top() - tile_height / 2.0)
                    .font_size(tile_width as u32 / 2)
                    .color(nannou::color::BLACK);
            }

            if black_bar > 0 {
                draw.ellipse()
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.bottom() + tile_height / 2.05)
                    .w_h(tile_width * 0.7, tile_width * 0.7)
                    .color(nannou::color::RED);
                draw.text(&black_bar.to_string())
                    .x_y(board_rect.left() + i as f32 * tile_width + tile_width / 2.0, board_rect.bottom() + tile_height / 2.0)
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

        if checkers != 0 {
            let color = if checkers > 0 { nannou::color::WHITE } else { nannou::color::RED };
            draw.ellipse()
                .x_y(x + tile_width / 2.0, y - tile_height / 6.2)
                .w_h(tile_width * 0.7, tile_width * 0.7)
                .color(color);

            let color = if checkers > 0 { nannou::color::BLACK } else { nannou::color::WHITE };
            draw.text(&checkers.abs().to_string())
                .x_y(x + tile_width / 2.0, y - tile_height / 7.0)
                .font_size(tile_width as u32 / 2)
                .color(color);
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
        if checkers != 0 {
            let color = if checkers > 0 { nannou::color::WHITE } else { nannou::color::RED };
            draw.ellipse()
                .x_y(x + tile_width / 2.0, y + tile_height / 7.8)
                .w_h(tile_width * 0.7, tile_width * 0.7)
                .color(color);

            let color = if checkers > 0 { nannou::color::BLACK } else { nannou::color::WHITE };
            draw.text(&checkers.abs().to_string())
                .x_y(x + tile_width / 2.0, y + tile_height / 7.0)
                .font_size(tile_width as u32 / 2)
                .color(color);
        }
    }

    // Draw dice 
    if let Some(dice) = model.current_dice {
        match model.board.active_player() {
            Player::White => {
                draw.text(&dice.to_string())
                    .x_y(-board_rect.w() / 4.0 + board_rect.x(), board_rect.h() / 40.0)
                    .font_size(board_rect.w() as u32 / 10)
                    .color(nannou::color::WHITE);
            }
            Player::Black => {
                draw.text(&dice.to_string())
                    .x_y(board_rect.w() / 4.0 + board_rect.x(), board_rect.h() / 40.0)
                    .font_size(board_rect.w() as u32 / 10)
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
                        Player::White => find_best_move(&board, dice, 2),
                        Player::Black => find_best_move(&board, dice, 1)
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