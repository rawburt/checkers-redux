use clap::Parser;
use runner::Runner;
use std::collections::HashMap;
use uuid::Uuid;

mod checkers;
mod human;
mod minimax;
mod runner;

use checkers::{Board, Player};
use human::MovementMap;

use crate::minimax::MinimaxContext;

const DRAW_LIMIT: u32 = 40;

fn game_loop(mut player1: Runner, mut player2: Runner, gameid: &Uuid) {
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

#[derive(Parser)]
struct Cli {
    /// Enable Alpha-Beta Pruning
    #[arg(short, long)]
    alpha_beta: bool,
    /// Enable the use of a Transposition Table with Alpha-Beta Pruning
    #[arg(short, long)]
    transposition_table: bool,
    /// Play against the AI
    #[arg(short, long)]
    play: bool,
    /// AI search depth limit
    #[arg(short, long, default_value_t = 6)]
    depth: u32,
    /// How many games to simulate
    #[arg(short, long, default_value_t = 1)]
    games: u32,
}

fn main() {
    let cli = Cli::parse();

    let ctx = MinimaxContext {
        table: cli.transposition_table,
        depth: cli.depth,
        alpha_beta: cli.alpha_beta || cli.transposition_table,
    };

    if cli.play {
        let mut table = HashMap::new();

        let gameid = Uuid::new_v4();
        println!("game.{}.ai.table = {}", &gameid, ctx.table);
        println!("game.{}.ai.depth = {}", &gameid, ctx.depth);
        println!("game.{}.ai.alpha_beta = {}", &gameid, ctx.alpha_beta);

        let player1 = Runner::human(MovementMap::new());
        let player2 = Runner::ai(ctx, &mut table);

        game_loop(player1, player2, &gameid);
    } else {
        let mut table = HashMap::new();

        for _ in 0..cli.games {
            let gameid = Uuid::new_v4();
            println!("game.{}.ai.table = {}", &gameid, ctx.table);
            println!("game.{}.ai.depth = {}", &gameid, ctx.depth);
            println!("game.{}.ai.alpha_beta = {}", &gameid, ctx.alpha_beta);

            let player1 = Runner::ai(ctx, &mut table);
            let player2 = Runner::random();

            game_loop(player1, player2, &gameid);
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
