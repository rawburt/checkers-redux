use std::collections::HashMap;

use crate::checkers::{Board, Movement, Player, Square, VALID_SQUARES};

const BACK_ROW: [usize; 8] = [5, 6, 7, 8, 37, 38, 39, 40];
fn evaluate(player: Player, board: &Board) -> i32 {
    let mut pawns = 0;
    let mut kings = 0;
    let mut back_row = 0;
    let mut total = 0;
    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            total += 1;
            if piece.get_player() == player {
                if piece.is_king() {
                    kings += 1;
                } else {
                    pawns += 1;
                }
            } else if piece.is_king() {
                kings -= 1;
            } else {
                pawns -= 1;
            }
        }
    }
    if total > 16 {
        for id in BACK_ROW {
            if let Square::Taken(piece) = board.get(id) {
                if piece.get_player() == player && !piece.is_king() {
                    back_row += 1;
                } else if !piece.is_king() {
                    back_row -= 1;
                }
            }
        }
    }
    (2 * pawns) + (5 * kings) + back_row
}

enum Flag {
    ExactValue,
    LowerBound,
    UpperBound,
}

// Ch2 The Transposition Table
// https://breukerd.home.xs4all.nl/thesis/
pub struct TTEntry {
    score: i32,
    depth: u8,
    flag: Flag,
}

const MAX: i32 = i32::MAX - 1;
const MIN: i32 = i32::MIN + 1;

fn negamax(
    player: Player,
    board: &mut Board,
    table: &mut Option<HashMap<Board, TTEntry>>,
    stats: &mut Stats,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    let old_alpha = alpha;

    if let Some(table) = table {
        if let Some(entry) = table.get(board) {
            stats.entry_hits += 1;
            if entry.depth >= depth {
                match entry.flag {
                    Flag::ExactValue => {
                        stats.table_used += 1;
                        return entry.score;
                    }
                    Flag::LowerBound => alpha = alpha.max(entry.score),
                    Flag::UpperBound => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    stats.table_used += 1;
                    return entry.score;
                }
            }
        }
    }

    if depth == 0 {
        return evaluate(player, board);
    }

    let mut value = MIN;

    for m in board.movements(player) {
        stats.explored += 1;
        board.do_movement(&m);
        value = value.max(-negamax(
            player.other(),
            board,
            table,
            stats,
            depth - 1,
            -beta,
            -alpha,
        ));
        board.undo_movement(&m);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }

    if let Some(table) = table {
        let flag = if value <= old_alpha {
            Flag::UpperBound
        } else if value >= beta {
            Flag::LowerBound
        } else {
            Flag::ExactValue
        };

        table.insert(
            *board,
            TTEntry {
                score: value,
                depth,
                flag,
            },
        );
    }

    value
}

// "Artificial Intelligence: A Modern Approach, Third Edition" by Stuary Russell and Peter Norvig
// -- 5.2.1 The minimax algorithm
fn minimax(
    player: Player,
    board: &mut Board,
    depth: u8,
    maximizing: bool,
    stats: &mut Stats,
) -> i32 {
    if depth == 0 {
        let maximizing_player = if maximizing { player } else { player.other() };
        return evaluate(maximizing_player, board);
    }
    if maximizing {
        let mut value = MIN;
        let movements = board.movements(player);
        for m in movements {
            stats.explored += 1;
            board.do_movement(&m);
            value = value.max(minimax(player.other(), board, depth - 1, false, stats));
            board.undo_movement(&m);
        }
        value
    } else {
        let mut value = MAX;
        let movements = board.movements(player);
        for m in movements {
            stats.explored += 1;
            board.do_movement(&m);
            value = value.min(minimax(player.other(), board, depth - 1, true, stats));
            board.undo_movement(&m);
        }
        value
    }
}

pub fn search(
    player: Player,
    board: &mut Board,
    alpha_beta: bool,
    table: &mut Option<HashMap<Board, TTEntry>>,
    depth: u8,
    stats: &mut Stats,
) -> Option<Movement> {
    let movements = board.movements(player);

    if movements.is_empty() {
        return None;
    }

    let mut value = MIN - 1;
    let mut movement = None;

    for m in movements {
        stats.explored += 1;
        board.do_movement(&m);
        let v = if alpha_beta {
            -negamax(player.other(), board, table, stats, depth, MIN, MAX)
        } else {
            minimax(player.other(), board, depth, false, stats)
        };
        board.undo_movement(&m);
        if v > value {
            movement = Some(m);
            value = v;
        }
    }

    movement
}

#[derive(Debug)]
pub struct Stats {
    pub explored: u32,
    pub entry_hits: u32,
    pub table_used: u32,
    pub moves: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            explored: 0,
            entry_hits: 0,
            table_used: 0,
            moves: 0,
        }
    }

    pub fn reset(&mut self) {
        self.explored = 0;
        self.entry_hits = 0;
        self.table_used = 0;
        self.moves = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_negamax_is_same_as_minimax() {
        let mut board1 = Board::new();
        let mut move_list_1 = Vec::new();
        let mut stats = Stats::new();
        loop {
            if let Some(movement) =
                search(Player::Player1, &mut board1, true, &mut None, 6, &mut stats)
            {
                board1.do_movement(&movement);
                move_list_1.push(movement);
            } else {
                break;
            }
            board1.mark_kings();
            if let Some(movement) = search(
                Player::Player2,
                &mut board1,
                false,
                &mut None,
                6,
                &mut stats,
            ) {
                board1.do_movement(&movement);
                move_list_1.push(movement);
            } else {
                break;
            }
            board1.mark_kings();
        }

        stats.reset();
        let mut board2 = Board::new();
        let mut move_list_2 = Vec::new();
        let mut table = Some(HashMap::new());
        loop {
            if let Some(movement) = search(
                Player::Player1,
                &mut board2,
                true,
                &mut table,
                6,
                &mut stats,
            ) {
                board2.do_movement(&movement);
                move_list_2.push(movement);
            } else {
                break;
            }
            board2.mark_kings();
            if let Some(movement) =
                search(Player::Player2, &mut board2, true, &mut None, 6, &mut stats)
            {
                board2.do_movement(&movement);
                move_list_2.push(movement);
            } else {
                break;
            }
            println!("{}", &board2);
            board2.mark_kings();
        }

        assert_eq!(move_list_1, move_list_2);
    }
}
