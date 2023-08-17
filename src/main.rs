use clap::Parser;
use rand::prelude::SliceRandom;
use std::collections::HashMap;

mod ai;
mod checkers;

use ai::{search, Stats, TTEntry};
use checkers::{Board, Player};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    alpha_beta: bool,
    #[arg(short, long)]
    transposition_table: bool,
    #[arg(short, long, default_value_t = 1)]
    games: u32,
    #[arg(short, long, default_value_t = 6)]
    depth: u8,
}

fn main() {
    let cli = Cli::parse();

    let mut player1 = 0;
    let mut player2 = 0;

    let mut stats = Stats::new();

    let mut table: Option<HashMap<Board, TTEntry>> = None;
    if cli.transposition_table {
        table = Some(HashMap::new());
    }

    for _ in 0..cli.games {
        let mut board = Board::new();
        let loser;
        loop {
            // PLAYER 1
            if let Some(movement) = search(
                Player::Player1,
                &mut board,
                cli.alpha_beta,
                &mut table,
                cli.depth,
                &mut stats,
            ) {
                board.do_movement(&movement);
                stats.moves += 1;
            } else {
                loser = Player::Player1;
                break;
            }
            board.mark_kings();

            // PLAYER 2
            let movements = board.movements(Player::Player2);
            if movements.is_empty() {
                loser = Player::Player2;
                break;
            }
            if let Some(movement) = movements.choose(&mut rand::thread_rng()) {
                board.do_movement(movement);
                stats.moves += 1;
            } else {
                panic!();
            }
            board.mark_kings();
        }
        match loser {
            Player::Player1 => player2 += 1,
            Player::Player2 => player1 += 1,
        };
        dbg!(&stats);
        println!("{}", board);
        stats.reset();
    }
    dbg!(player1);
    dbg!(player2);
}
