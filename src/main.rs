use clap::Parser;
use runner::Runner;
use std::collections::HashMap;

mod checkers;
mod human;
mod minimax;
mod runner;

use checkers::{Board, Player};
use human::MovementMap;

use crate::minimax::MinimaxContext;

fn game_loop(mut player1: Runner, mut player2: Runner) {
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

        if draw >= 40 {
            break;
        }
    }

    match winner {
        None => println!("draw"),
        Some(Player::Player1) => println!("player 1"),
        Some(Player::Player2) => println!("player 2"),
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    alpha_beta: bool,
    #[arg(short, long)]
    transposition_table: bool,
    #[arg(short, long)]
    play: bool,
    #[arg(short, long, default_value_t = 6)]
    depth: u32,
}

fn main() {
    let cli = Cli::parse();

    let ctx = MinimaxContext {
        table: cli.transposition_table,
        depth: cli.depth,
        alpha_beta: cli.alpha_beta,
        time: None,
    };

    if cli.play {
        let player1 = Runner::human(MovementMap::new());
        let player2 = Runner::ai(ctx, HashMap::new());

        game_loop(player1, player2);
    } else {
        let player1 = Runner::ai(ctx, HashMap::new());
        let player2 = Runner::random();

        game_loop(player1, player2);
    }
}
