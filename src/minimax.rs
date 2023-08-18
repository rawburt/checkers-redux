use std::collections::HashMap;

use crate::checkers::{Board, Movement, Player, Square, VALID_SQUARES};

const CENTER: [usize; 6] = [15, 16, 20, 21, 24, 25];
const BACKP1: [usize; 4] = [5, 6, 7, 8];
const BACKP2: [usize; 4] = [37, 38, 39, 40];

pub fn evaluation1(board: &Board, player: Player) -> i32 {
    let mut pawn = 0;
    let mut king = 0;
    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            if piece.get_player() == player {
                if piece.is_king() {
                    king += 1;
                } else {
                    pawn += 1;
                }
            } else if piece.is_king() {
                king -= 1;
            } else {
                pawn -= 1;
            }
        }
    }
    pawn + (3 * king)
}

pub fn evaluation2(board: &Board, player: Player) -> i32 {
    let mut me = 0;
    let mut you = 0;
    let mut tempo = 0;
    let mut defense = 0;
    let mut pawns = 0;
    let mut kings = 0;
    let mut total = 0;
    let mut kcent = 0;
    let mut cramp = 0;
    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            total += 1;
            if piece.get_player() == player {
                me += 1;
                if piece.is_king() {
                    kings += 1;
                    if CENTER.contains(&id) {
                        kcent += 1;
                    }
                } else {
                    pawns += 1;
                    match player {
                        Player::Player1 => {
                            if BACKP1.contains(&id) {
                                defense += 1;
                            }
                        }
                        Player::Player2 => {
                            if BACKP2.contains(&id) {
                                defense += 1;
                            }
                        }
                    };
                    if player == Player::Player1 && id >= 28 {
                        tempo += 1;
                    }
                    if player == Player::Player2 && id <= 17 {
                        tempo += 1;
                    }
                }
            } else if piece.is_king() {
                you += 1;
                kings -= 1;
                if CENTER.contains(&id) {
                    kcent -= 1;
                }
            } else {
                you += 1;
                pawns -= 1;
                if player == Player::Player1 && id <= 17 {
                    tempo -= 1;
                }
                if player == Player::Player2 && id >= 28 {
                    tempo -= 1;
                }
                match player.other() {
                    Player::Player1 => {
                        if BACKP1.contains(&id) {
                            defense -= 1;
                        }
                    }
                    Player::Player2 => {
                        if BACKP2.contains(&id) {
                            defense -= 1;
                        }
                    }
                };
            }
        }
    }

    if let Square::Taken(piece1) = board.get(23) {
        if piece1.get_player() == Player::Player1 {
            if let Square::Taken(piece2) = board.get(28) {
                if piece1.get_player() != piece2.get_player() {
                    if player == Player::Player1 {
                        cramp += 1;
                    } else {
                        cramp -= 1;
                    }
                }
            }
        }
    }

    if let Square::Taken(piece1) = board.get(22) {
        if piece1.get_player() == Player::Player2 {
            if let Square::Taken(piece2) = board.get(17) {
                if piece1.get_player() != piece2.get_player() {
                    if player == Player::Player1 {
                        cramp -= 1;
                    } else {
                        cramp += 1;
                    }
                }
            }
        }
    }

    let d = if total <= 12 { -10 } else { 15 };
    let t = if total <= 16 { 10 } else { 40 };

    // dbg!(pawns, kings, defense, tempo, me, you, kcent, cramp);

    (105 * pawns)
        + (125 * kings)
        + (d * defense)
        + (t * tempo)
        + ((250 * (me - you)) / (me + you))
        + (me - you)
        + (30 * kcent)
        + (10 * cramp)
}

pub struct Stats {
    pub moves: u32,
    pub explored: u32,
    pub beta_cuts: u32,
    pub tt_exact: u32,
    pub tt_cuts: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            moves: 0,
            explored: 0,
            beta_cuts: 0,
            tt_exact: 0,
            tt_cuts: 0,
        }
    }
}

enum Flag {
    Exact,
    Lowerbound,
    Upperbound,
}

pub struct TTEntry {
    movement: Movement,
    score: i32,
    depth: u32,
    flag: Flag,
}

struct MinimaxResult {
    score: i32,
    movement: Option<Movement>,
}

#[derive(Clone, Copy)]
pub struct MinimaxContext {
    pub table: bool,
    pub depth: u32,
    pub alpha_beta: bool,
    pub quiescence: bool,
    pub verbose: bool,
    pub heuristic: fn(&Board, Player) -> i32,
}

#[allow(clippy::too_many_arguments)]
fn minimax(
    stats: &mut Stats,
    ctx: &MinimaxContext,
    board: &mut Board,
    player: Player,
    table: &mut HashMap<u128, TTEntry>,
    mut depth: u32,
    mut alpha: i32,
    mut beta: i32,
) -> MinimaxResult {
    let alpha_orig = alpha;
    let mut best_move: Option<Movement> = None;
    let movements = board.movements(player);

    if depth == 0 && ctx.quiescence && !movements.is_empty() && movements[0].is_jump() {
        depth = 1;
    }

    if depth == 0 || movements.is_empty() {
        let result = MinimaxResult {
            score: (ctx.heuristic)(board, player),
            movement: best_move,
        };
        return result;
    }

    if ctx.table {
        if let Some(entry) = table.get(&board.hash()) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        stats.tt_exact += 1;
                        return MinimaxResult {
                            score: entry.score,
                            movement: Some(entry.movement.clone()),
                        };
                    }
                    Flag::Lowerbound => {
                        if alpha < entry.score {
                            alpha = entry.score;
                        }
                    }
                    Flag::Upperbound => {
                        if beta > entry.score {
                            beta = entry.score;
                        }
                    }
                }
                if alpha >= beta {
                    stats.tt_cuts += 1;
                    return MinimaxResult {
                        score: entry.score,
                        movement: Some(entry.movement.clone()),
                    };
                }
            }
        }
    }

    let mut value = i32::MIN;

    for m in movements {
        stats.explored += 1;
        board.do_movement(&m);
        let score = -minimax(
            stats,
            ctx,
            board,
            player.other(),
            table,
            depth - 1,
            -beta,
            -alpha,
        )
        .score;
        board.undo_movement(&m);
        if value < score {
            value = score;
            best_move = Some(m);
            if value >= beta && ctx.alpha_beta {
                stats.beta_cuts += 1;
                break;
            }
        }
        if alpha < value {
            alpha = value;
        }
    }

    if ctx.table {
        if let Some(m) = &best_move {
            let flag = if value <= alpha_orig {
                Flag::Upperbound
            } else if value >= beta {
                Flag::Lowerbound
            } else {
                Flag::Exact
            };
            table.insert(
                board.hash(),
                TTEntry {
                    movement: m.clone(),
                    score: value,
                    depth,
                    flag,
                },
            );
        }
    }

    MinimaxResult {
        score: value,
        movement: best_move,
    }
}

pub fn get_movement(
    stats: &mut Stats,
    ctx: &MinimaxContext,
    board: &mut Board,
    player: Player,
    table: &mut HashMap<u128, TTEntry>,
) -> Option<Movement> {
    let movements = board.movements(player);

    if movements.is_empty() {
        return None;
    }

    let mut best_movement: Option<Movement> = None;
    let mut best_score = None;

    for d in 1..=ctx.depth {
        let result = minimax(
            stats,
            ctx,
            board,
            player,
            table,
            d,
            i32::MIN + 1,
            i32::MAX - 1,
        );
        if let Some(m) = result.movement {
            best_movement = Some(m);
            best_score = Some(result.score);
        }
    }

    if ctx.verbose {
        if best_score.is_some() {
            println!("minimax engine score: {}", best_score.unwrap());
        } else {
            println!("no score found");
        }
    }

    if best_movement.is_some() {
        stats.moves += 1;
    }

    best_movement
}
