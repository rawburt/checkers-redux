use clap::Parser;
use rand::prelude::SliceRandom;
use std::collections::HashMap;

mod checkers;
mod minimax;

use checkers::{Board, Movement, Player, Square, SquareState};

use crate::minimax::{get_movement, MinimaxContext, TTEntry};

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

impl Default for MovementMap {
    fn default() -> Self {
        MovementMap::new()
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

    if steps.len() < 3 {
        return None;
    }

    match steps[0] {
        "S:" => {
            let start = map.get(steps[1])?;
            let end = map.get(steps[2])?;
            if let Square::Taken(piece) = board.get(*start) {
                let square_start = SquareState::piece(*start, piece);
                let square_end = SquareState::empty(*end);
                return Some(Movement::simple(square_start, square_end));
            }
            None
        }
        "J:" => parse_jump(board, map, &steps, 1),
        "M:" => {
            let mut jump = parse_jump(board, map, &steps, 2)?;
            parse_multi_jump(board, map, &steps, 5, &mut jump);
            Some(jump)
        }
        _ => None,
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

enum RunnerKind {
    Random,
    AI,
    Human,
}

struct Runner {
    kind: RunnerKind,
    context: Option<MinimaxContext>,
    table: Option<HashMap<Board, TTEntry>>,
    map: Option<MovementMap>,
}

impl Runner {
    pub fn random() -> Self {
        Self {
            kind: RunnerKind::Random,
            context: None,
            table: None,
            map: None,
        }
    }

    pub fn ai(context: MinimaxContext, table: HashMap<Board, TTEntry>) -> Self {
        Self {
            kind: RunnerKind::AI,
            context: Some(context),
            table: Some(table),
            map: None,
        }
    }

    pub fn human(map: MovementMap) -> Self {
        Self {
            kind: RunnerKind::Human,
            context: None,
            table: None,
            map: Some(map),
        }
    }

    pub fn get_move(&mut self, board: &mut Board, player: Player) -> Option<Movement> {
        match self.kind {
            RunnerKind::Random => {
                let movements = board.movements(player);
                if movements.is_empty() {
                    return None;
                }
                movements.choose(&mut rand::thread_rng()).cloned()
            }
            RunnerKind::AI => get_movement(
                self.context.as_ref().unwrap(),
                board,
                player,
                self.table.as_mut().unwrap(),
            ),
            RunnerKind::Human => {
                let movements = board.movements(Player::Player1);
                if movements.is_empty() {
                    return None;
                }
                println!("{}", &board);
                loop {
                    let movement = get_user_input(board, self.map.as_ref().unwrap());
                    if let Some(movement) = movement {
                        if movements.iter().any(|m| *m == movement) {
                            return Some(movement);
                        }
                    }
                }
            }
        }
    }
}

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
