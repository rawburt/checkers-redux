use std::collections::HashMap;

use crate::checkers::{Board, Movement, Player, Square, VALID_SQUARES};

fn get_score(board: &Board, player: Player) -> i32 {
    let mut score = 0;
    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            if piece.get_player() == player {
                if piece.is_king() {
                    score += 3;
                } else {
                    score += 1;
                }
            } else if piece.is_king() {
                score -= 3;
            } else {
                score -= 1;
            }
        }
    }
    score
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

pub struct MinimaxContext {
    pub table: bool,
    pub depth: u32,
    pub alpha_beta: bool,
    pub time: Option<u32>,
}

fn minimax(
    ctx: &MinimaxContext,
    board: &mut Board,
    player: Player,
    table: &mut HashMap<Board, TTEntry>,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
) -> MinimaxResult {
    let mut best_move: Option<Movement> = None;
    let movements = board.movements(player);

    if depth == 0 || movements.is_empty() {
        let result = MinimaxResult {
            score: get_score(board, player),
            movement: best_move,
        };
        return result;
    }

    if ctx.table {
        if let Some(entry) = table.get(board) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        return MinimaxResult {
                            score: entry.score,
                            movement: Some(entry.movement.clone()),
                        }
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
        board.do_movement(&m);
        let score = -minimax(ctx, board, player.other(), table, depth - 1, -beta, -alpha).score;
        board.undo_movement(&m);
        if value < score {
            value = score;
            best_move = Some(m);
            if value >= beta && ctx.alpha_beta {
                break;
            }
        }
        if alpha < value {
            alpha = value;
        }
    }

    if ctx.table {
        if let Some(m) = &best_move {
            let flag = if value <= alpha {
                Flag::Upperbound
            } else if value >= beta {
                Flag::Lowerbound
            } else {
                Flag::Exact
            };
            table.insert(
                *board,
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
    ctx: &MinimaxContext,
    board: &mut Board,
    player: Player,
    table: &mut HashMap<Board, TTEntry>,
) -> Option<Movement> {
    let movements = board.movements(player);
    if movements.is_empty() {
        return None;
    }
    let mut best_movement: Option<Movement> = None;
    for d in 1..=ctx.depth {
        let result = minimax(ctx, board, player, table, d, i32::MIN + 1, i32::MAX - 1);
        if let Some(m) = result.movement {
            best_movement = Some(m);
        }
    }
    best_movement
}
