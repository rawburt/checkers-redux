use clap::Parser;
use rand::prelude::SliceRandom;
use std::{collections::HashMap, hash::Hash};

mod ai;
mod checkers;
mod minimax;

use ai::{search, Stats};
use checkers::{Board, Movement, Player, Square, SquareState};

use crate::minimax::{get_movement, TTEntry};

pub struct MovementMap {
    pub map: HashMap<String, usize>,
}

impl MovementMap {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("A8".to_string(), 5);
        map.insert("C8".to_string(), 6);
        map.insert("E8".to_string(), 7);
        map.insert("G8".to_string(), 8);

        map.insert("B7".to_string(), 10);
        map.insert("D7".to_string(), 11);
        map.insert("F7".to_string(), 12);
        map.insert("H7".to_string(), 13);

        map.insert("A6".to_string(), 14);
        map.insert("C6".to_string(), 15);
        map.insert("E6".to_string(), 16);
        map.insert("G6".to_string(), 17);

        map.insert("B5".to_string(), 19);
        map.insert("D5".to_string(), 20);
        map.insert("F5".to_string(), 21);
        map.insert("H5".to_string(), 22);

        map.insert("A4".to_string(), 23);
        map.insert("C4".to_string(), 24);
        map.insert("E4".to_string(), 25);
        map.insert("G4".to_string(), 26);

        map.insert("B3".to_string(), 28);
        map.insert("D3".to_string(), 29);
        map.insert("F3".to_string(), 30);
        map.insert("H3".to_string(), 31);

        map.insert("A2".to_string(), 32);
        map.insert("C2".to_string(), 33);
        map.insert("E2".to_string(), 34);
        map.insert("G2".to_string(), 35);

        map.insert("B1".to_string(), 37);
        map.insert("D1".to_string(), 38);
        map.insert("F1".to_string(), 39);
        map.insert("H1".to_string(), 40);

        Self { map }
    }

    fn get(&self, key: &str) -> Option<&usize> {
        self.map.get(key)
    }
}

pub fn parse_jump(
    board: &Board,
    map: &MovementMap,
    steps: &[&str],
    idx: usize,
) -> Option<Movement> {
    if steps.len() <= idx + 2 {
        return None;
    }
    let start = map.get(steps[idx])?;
    let jumped = map.get(steps[idx + 1])?;
    let end = map.get(steps[idx + 2])?;
    if let Square::Taken(start_piece) = board.get(*start) {
        if let Square::Taken(jumped_piece) = board.get(*jumped) {
            let square_start = SquareState::piece(*start, start_piece);
            let square_jumped = SquareState::piece(*jumped, jumped_piece);
            let square_end = SquareState::empty(*end);
            return Some(Movement::jump(square_start, square_end, square_jumped));
        }
    }
    None
}

pub fn parse_multi_jump(
    board: &Board,
    map: &MovementMap,
    steps: &Vec<&str>,
    idx: usize,
    parent: &mut Movement,
) {
    if steps.len() <= idx {
        return;
    }
    if steps[idx] != "J:" {
        panic!("expected jump 1");
    }
    match parse_jump(board, map, steps, idx + 1) {
        None => panic!("expected jump 2"),
        Some(mut m) => {
            parse_multi_jump(board, map, steps, idx + 4, &mut m);
            parent.set_next(&m);
        }
    }
}

fn get_user_input(board: &Board, map: &MovementMap) -> Option<Movement> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let steps: Vec<&str> = line.trim().split(' ').collect();
    // dbg!(&steps);

    if steps.len() < 3 {
        return None;
    }

    match steps[0] {
        "S:" => {
            let start = map.get(steps[1])?;
            let end = map.get(steps[2])?;
            // dbg!(start);
            // dbg!(end);
            if let Square::Taken(piece) = board.get(*start) {
                let square_start = SquareState::piece(*start, piece);
                let square_end = SquareState::empty(*end);
                return Some(Movement::simple(square_start, square_end));
            }
            return None;
        }
        "J:" => return parse_jump(board, map, &steps, 1),
        "M:" => {
            let mut jump = parse_jump(board, map, &steps, 2)?;
            parse_multi_jump(board, map, &steps, 5, &mut jump);
            Some(jump)
        }
        _ => return None,
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
    #[arg(short, long, default_value_t = 1)]
    games: u32,
    #[arg(short, long, default_value_t = 6)]
    depth: u8,
}

fn main() {
    let mut cli = Cli::parse();

    let mut player1 = 0;
    let mut player2 = 0;

    let mut stats = Stats::new();

    // let mut table: Option<HashMap<(Player, Board), TTEntry>> = None;
    // if cli.transposition_table {
    //     table = Some(HashMap::new());
    // }

    if cli.play {
        // let map = MovementMap::new();
        // let mut board = Board::new();
        // let loser;
        // loop {
        //     // PLAYER 1 human
        //     let movements = board.movements(Player::Player1);
        //     if movements.is_empty() {
        //         loser = Player::Player1;
        //         break;
        //     }
        //     println!("{}", &board);
        //     loop {
        //         let movement = get_user_input(&board, &map);
        //         // dbg!(&movement);
        //         if let Some(movement) = movement {
        //             // dbg!(&movement);
        //             // dbg!(&movements);
        //             if movements.iter().any(|m| *m == movement) {
        //                 board.do_movement(&movement);
        //                 break;
        //             }
        //         }
        //     }
        //     stats.moves += 1;

        //     // PLAYER 2
        //     if let Some(movement) = search(
        //         Player::Player2,
        //         &mut board,
        //         cli.alpha_beta,
        //         &mut table,
        //         cli.depth,
        //         &mut stats,
        //     ) {
        //         board.do_movement(&movement);
        //         stats.moves += 1;
        //     } else {
        //         loser = Player::Player1;
        //         break;
        //     }
        //     board.mark_kings();
        // }
        // match loser {
        //     Player::Player1 => player2 += 1,
        //     Player::Player2 => player1 += 1,
        // };
        // dbg!(&stats);
        // println!("{}", board);
    } else {
        for _ in 0..cli.games {
            let mut board = Board::new();
            let loser;
            let mut ttable: Option<HashMap<Board, TTEntry>> = None;
            if cli.transposition_table {
                ttable = Some(HashMap::new());
                cli.alpha_beta = true;
            }
            loop {
                // PLAYER 1
                if let Some(movement) =
                    get_movement(&mut board, Player::Player1, &mut ttable, cli.alpha_beta, 6)
                {
                    board.do_movement(&movement);
                    stats.moves += 1;
                } else {
                    loser = Player::Player1;
                    break;
                }
                board.mark_kings();

                // if let Some(movement) = search(
                //     Player::Player1,
                //     &mut board,
                //     cli.alpha_beta,
                //     &mut table,
                //     cli.depth,
                //     &mut stats,
                // ) {
                //     board.do_movement(&movement);
                //     stats.moves += 1;
                // } else {
                //     loser = Player::Player1;
                //     break;
                // }
                // board.mark_kings();

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
    }

    dbg!(player1);
    dbg!(player2);
}
