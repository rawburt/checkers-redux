use rand::prelude::SliceRandom;
use clap::Parser;
mod checkers;
mod ai;

use checkers::{Player, Board};
use ai::{search, Stats};

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

    for _ in 0..cli.games {
        let mut board = Board::new();
        let loser;
        loop {
            // PLAYER 1
            if let Some(movement) = search(
                Player::Player1,
                &mut board,
                cli.alpha_beta,
                cli.depth,
                &mut stats,
            ) {
                board.do_movement(&movement);
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
            } else {
                panic!();
            }
            board.mark_kings();
        }
        match loser {
            Player::Player1 => player2 += 1,
            Player::Player2 => player1 += 1,
        };
        dbg!("{}", &stats);
        stats.reset();
    }
    dbg!(player1);
    dbg!(player2);
}
