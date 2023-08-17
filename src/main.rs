use clap::{Parser, ValueEnum};
use minimax::{evaluation1, evaluation2, MinimaxContext};
use runner::Runner;
use std::collections::HashMap;
use uuid::Uuid;

mod checkers;
mod human;
mod minimax;
mod runner;

use checkers::{Board, Player};
use human::MovementMap;

const DRAW_LIMIT: u32 = 40;

fn game_loop(mut player1: Runner, mut player2: Runner, gameid: &Uuid, verbose: bool) {
    let mut board = Board::new();
    let mut draw = 0;
    let mut winner: Option<Player> = None;
    loop {
        // PLAYER 1
        if let Some(movement) = player1.get_move(&mut board, Player::Player1) {
            board.do_movement(&movement);
            if movement.is_jump() {
                draw = 0;
            } else {
                draw += 1;
            }
            if board.mark_kings() > 0 {
                draw = 0;
            }
        } else {
            winner = Some(Player::Player2);
            break;
        }

        if verbose {
            println!("{}", &board);
        }

        // PLAYER 2
        if let Some(movement) = player2.get_move(&mut board, Player::Player2) {
            board.do_movement(&movement);
            if movement.is_jump() {
                draw = 0;
            } else {
                draw += 1;
            }
            if board.mark_kings() > 0 {
                draw = 0;
            }
        } else {
            winner = Some(Player::Player1);
            break;
        }

        if verbose {
            println!("{}", &board);
        }

        if draw >= DRAW_LIMIT {
            break;
        }
    }

    match winner {
        None => println!("game.{}.winner = draw", &gameid),
        Some(Player::Player1) => println!("game.{}.winner = player 1", &gameid),
        Some(Player::Player2) => println!("game.{}.winner = player 2", &gameid),
    }

    player1.display_stats("player1", gameid);
    player2.display_stats("player2", gameid);
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Engine {
    AI,
    Random,
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::AI => write!(f, "ai"),
            Engine::Random => write!(f, "random"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Eval {
    V1,
    V2,
}

impl std::fmt::Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Eval::V1 => write!(f, "v1"),
            Eval::V2 => write!(f, "v2"),
        }
    }
}

impl Eval {
    fn to_fn(&self) -> fn(&Board, Player) -> i32 {
        match self {
            Eval::V1 => evaluation1,
            Eval::V2 => evaluation2,
        }
    }
}

#[derive(Parser)]
struct Cli {
    /// Player 1 engine
    #[arg(long, default_value = "ai")]
    p1_engine: Engine,
    /// Enable Alpha-Beta Pruning for Player 1
    #[arg(long)]
    p1_alpha_beta: bool,
    /// Enable the use of a Transposition Table with Alpha-Beta Pruning for Player 1
    #[arg(long)]
    p1_transposition_table: bool,
    /// Enable quiescence search for Player 1
    #[arg(long)]
    p1_quiescence: bool,
    /// AI search depth limit for Player 1
    #[arg(long, default_value_t = 6)]
    p1_depth: u32,
    /// Player 1 evaluation function
    #[arg(long, default_value = "v1")]
    p1_eval: Eval,
    /// Player 2 engine
    #[arg(long, default_value = "random")]
    p2_engine: Engine,
    /// Enable Alpha-Beta Pruning for Player 2
    #[arg(long)]
    p2_alpha_beta: bool,
    /// Enable the use of a Transposition Table with Alpha-Beta Pruning for Player 2
    #[arg(long)]
    p2_transposition_table: bool,
    /// Enable quiescence search for Player 2
    #[arg(long)]
    p2_quiescence: bool,
    /// AI search depth limit for Player 2
    #[arg(long, default_value_t = 6)]
    p2_depth: u32,
    /// Player 2 evaluation function
    #[arg(long, default_value = "v1")]
    p2_eval: Eval,
    /// You (Player 1) against the AI (Player 2)
    #[arg(long)]
    play: bool,
    /// How many games to simulate
    #[arg(short, long, default_value_t = 1)]
    games: u32,
    /// Show moves made by engines during simulation
    #[arg(short, long)]
    verbose: bool
}

fn display_cli_config(cli: &Cli) {
    println!("config.games = {}", cli.games);

    println!("config.player1.engine = {}", cli.p1_engine);
    println!("config.player1.alpha_beta = {}", cli.p1_alpha_beta);
    println!("config.player1.transposition_table = {}", cli.p1_transposition_table);
    println!("config.player1.quiescence = {}", cli.p1_quiescence);
    println!("config.player1.depth = {}", cli.p1_depth);
    println!("config.player1.eval = {}", cli.p1_eval);

    println!("config.player2.engine = {}", cli.p2_engine);
    println!("config.player2.alpha_beta = {}", cli.p2_alpha_beta);
    println!("config.player2.transposition_table = {}", cli.p2_transposition_table);
    println!("config.player1.quiescence = {}", cli.p2_quiescence);
    println!("config.player2.depth = {}", cli.p2_depth);
    println!("config.player2.eval = {}", cli.p2_eval);
}

fn main() {
    let cli = Cli::parse();

    display_cli_config(&cli);

    let ctx_p1 = MinimaxContext {
        table: cli.p1_transposition_table,
        depth: cli.p1_depth,
        alpha_beta: cli.p1_alpha_beta || cli.p1_transposition_table,
        quiescence: cli.p1_quiescence,
        heuristic: cli.p1_eval.to_fn(),
    };

    let ctx_p2 = MinimaxContext {
        table: cli.p2_transposition_table,
        depth: cli.p2_depth,
        alpha_beta: cli.p2_alpha_beta || cli.p2_transposition_table,
        quiescence: cli.p2_quiescence,
        heuristic: cli.p1_eval.to_fn(),
    };

    if cli.play {
        let mut table = HashMap::with_capacity(100_000);

        let gameid = Uuid::new_v4();

        let player1 = Runner::human(MovementMap::new());
        let player2 = match cli.p2_engine {
            Engine::AI => Runner::ai(ctx_p2, &mut table),
            Engine::Random => Runner::random(),
        };

        game_loop(player1, player2, &gameid, false);
    } else {
        let mut table1 = HashMap::with_capacity(100_000);
        let mut table2 = HashMap::with_capacity(100_000);

        for _ in 0..cli.games {
            let gameid = Uuid::new_v4();

            let player1 = match cli.p1_engine {
                Engine::AI => Runner::ai(ctx_p1, &mut table1),
                Engine::Random => Runner::random(),
            };
            let player2 = match cli.p2_engine {
                Engine::AI => Runner::ai(ctx_p2, &mut table2),
                Engine::Random => Runner::random(),
            };

            game_loop(player1, player2, &gameid, cli.verbose);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        checkers::{Piece, Square},
        human::parse_input,
        minimax::{get_movement, Stats},
    };

    use super::*;

    #[test]
    fn test_bugfix_1() {
        let ctx = MinimaxContext {
            table: false,
            depth: 6,
            alpha_beta: true,
            quiescence: false,
            heuristic: evaluation1,
        };
        let mut table = HashMap::new();

        let mut board = Board::empty();

        board.set(28, Square::Taken(Piece::player1_pawn()));
        board.set(8, Square::Taken(Piece::player1_pawn()));
        board.set(29, Square::Taken(Piece::player1_king()));
        board.set(24, Square::Taken(Piece::player1_king()));

        board.set(12, Square::Taken(Piece::player2_pawn()));
        board.set(26, Square::Taken(Piece::player2_pawn()));
        board.set(39, Square::Taken(Piece::player2_pawn()));
        board.set(40, Square::Taken(Piece::player2_pawn()));
        board.set(11, Square::Taken(Piece::player2_king()));

        let mut input = String::from("J: G8 F7 E6");
        let map = MovementMap::new();
        let movement = parse_input(&mut input, &board, &map);

        assert!(movement.is_some());

        let movement = movement.unwrap();
        let movements = board.movements(Player::Player1);

        assert!(movements.iter().any(|m| *m == movement));

        board.do_movement(&movement);

        let ai_movement = get_movement(
            &mut Stats::new(),
            &ctx,
            &mut board,
            Player::Player2,
            &mut table,
        );

        assert!(ai_movement.is_some());

        let ai_movement = ai_movement.unwrap();
        board.do_movement(&ai_movement);

        assert_eq!(board.get(21), Square::Taken(Piece::player2_king()));
    }
}
