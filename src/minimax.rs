// This module contains the data structures and functions used to implement Minimax and the
// various features and optimizations that the engine supports.

use std::{collections::HashMap, time::Instant};

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

// Advancement
const ADV_5_6: [usize; 8] = [23, 24, 25, 26, 28, 29, 30, 31];
const ADV_3_4: [usize; 8] = [14, 15, 16, 17, 19, 20, 21, 22];

// Move
const MOVE_SYSTEM: [usize; 16] = [5, 6, 7, 8, 14, 15, 16, 17, 23, 24, 25, 26, 32, 33, 34, 35];

// Based on heuristics described in Arthur L. Samuel's "Some Studies in Machine Learning Using the Game of Checkers" (1959)
pub fn evaluation3(board: &Board, player: Player) -> i32 {
    let mut mob = 0;
    let mut deny = 0;
    let mut center = 0;
    let mut king_center = 0;
    let mut mov = 0;
    let mut adv = 0;
    let mut back = 0;
    let mut thret = 0;
    let mut me_kings = 0;
    let mut me_pawns = 0;
    let mut you_kings = 0;
    let mut you_pawns = 0;

    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            // basic piece counts
            if piece.get_player() == player {
                if piece.is_king() {
                    me_kings += 1;
                } else {
                    me_pawns += 1;
                }
            } else if piece.is_king() {
                you_kings += 1;
            } else {
                you_pawns += 1;
            }
            // total mobility (can the piece move somewhere?)
            for m in piece.movements() {
                let id_to = (id as i32 + m) as usize;
                if let Square::Empty = board.get(id_to) {
                    if piece.get_player() == player {
                        mob += 1;
                    }

                    // denial of occupancy (will this movement allow capture for other player?)
                    for s in &[-5, -4, 4, 5] {
                        let id_surround = (id_to as i32 + s) as usize;
                        if let Square::Taken(surround_piece) = board.get(id_surround) {
                            if surround_piece.get_player() != player {
                                // where opponent will land on their jump
                                let id_jump_land = (id_to as i32 - s) as usize;
                                if let Square::Empty = board.get(id_jump_land) {
                                    // do i have any pieces that can jump back?
                                    // NAIVE: TODO: FIX
                                    for j in &[-5, -4, 4, 5] {
                                        let id_defend = (id_jump_land as i32 + j) as usize;
                                        if let Square::Taken(defender) = board.get(id_defend) {
                                            if defender.get_player() == player {
                                                deny += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // threat (does this movement threaten a capture?)
                    for j in piece.movements() {
                        let id_jump = (id_to as i32 + j) as usize;
                        if let Square::Taken(jumped_piece) = board.get(id_jump) {
                            if jumped_piece.get_player() != piece.get_player() {
                                let id_land = (id_jump as i32 + j) as usize;
                                if let Square::Empty = board.get(id_land) {
                                    if piece.get_player() == player {
                                        thret += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let me = (2 * me_pawns) + (3 * me_kings);
    let you = (2 * you_pawns) + (3 * you_kings);

    // Move
    if me < 25 && me == you {
        let mut count = 0;
        for id in MOVE_SYSTEM {
            if let Square::Taken(_) = board.get(id) {
                count += 1;
            }
        }
        if count % 2 == 0 {
            mov = 1;
        }
    }

    // Advancement
    for id in ADV_3_4 {
        if let Square::Taken(piece) = board.get(id) {
            if piece.get_player() == Player::Player2 {
                if player == Player::Player2 {
                    adv += 1;
                } else {
                    adv -= 1;
                }
            } else if player == Player::Player1 {
                adv -= 1;
            } else {
                adv += 1;
            }
        }
    }
    for id in ADV_5_6 {
        if let Square::Taken(piece) = board.get(id) {
            if piece.get_player() == Player::Player1 {
                if player == Player::Player1 {
                    adv += 1;
                } else {
                    adv -= 1;
                }
            } else if player == Player::Player2 {
                adv -= 1;
            } else {
                adv += 1;
            }
        }
    }

    // Back Row Bridge
    if me_kings == 0 {
        if let (Square::Taken(_), Square::Taken(_)) = (board.get(6), board.get(8)) {
            if player == Player::Player1 {
                back = 1;
            }
        }
        if let (Square::Taken(_), Square::Taken(_)) = (board.get(37), board.get(39)) {
            if player == Player::Player2 {
                back = 1;
            }
        }
    }

    // center control and king center
    for id in CENTER {
        if let Square::Taken(piece) = board.get(id) {
            center += 1;
            if piece.is_king() {
                king_center += 1;
            }
        }
    }

    let mobil = mob - deny;
    let undenied_mobility = mobil > 0;
    let total_mobility = mob > 0;
    let denial_of_occupancy = deny > 0;
    let control = center > 0;

    let demmo = if denial_of_occupancy && !total_mobility {
        1
    } else {
        0
    };
    let mode_2 = if undenied_mobility && !denial_of_occupancy {
        1
    } else {
        0
    };
    let mode_3 = if !undenied_mobility && denial_of_occupancy {
        1
    } else {
        0
    };
    let moc_2 = if !undenied_mobility && control { 1 } else { 0 };
    let moc_3 = if undenied_mobility && !control { 1 } else { 0 };
    let moc_4 = if !undenied_mobility && !control { 1 } else { 0 };

    let b: i32 = 2;

    (-moc_2 * b.pow(18))
        + (king_center * b.pow(16))
        + (-moc_4 * b.pow(14))
        + (-mode_3 * b.pow(13))
        + (-demmo * b.pow(11))
        + (mov * b.pow(8))
        + (-adv * b.pow(8))
        + (-mode_2 * b.pow(8))
        + (-back * b.pow(6))
        + (center * b.pow(5))
        + (thret * b.pow(5))
        + (moc_3 * b.pow(4))
        + ((me - you) * b.pow(20))
}

// Define the data structure used to collect stats about the performance of the Minimax algorithm.
pub struct Stats {
    pub moves: u32,
    pub explored: u32,
    pub beta_cuts: u32,
    pub tt_exact: u32,
    pub tt_cuts: u32,
    pub max_depth: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            moves: 0,
            explored: 0,
            beta_cuts: 0,
            tt_exact: 0,
            tt_cuts: 0,
            max_depth: 0,
        }
    }
}

// Define the flag states used in a [TTEntry].
enum Flag {
    Exact,
    Lowerbound,
    Upperbound,
}

// Define an entry in the Transposition Table.
pub struct TTEntry {
    // What movement was selected for the given board position.
    movement: Movement,
    // What was the evaluation function score.
    score: i32,
    // How deep was the evaluation conducted at.
    depth: u32,
    // The flag used for the Alpha-Beta state of the table entry.
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
    pub iterative: bool,
    pub verbose: bool,
    pub heuristic: fn(&Board, Player) -> i32,
}

#[allow(clippy::too_many_arguments)]
fn minimax(
    stats: &mut Stats,
    ctx: &MinimaxContext,
    mut max_depth: u32,
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

    max_depth += 1;
    if stats.max_depth < max_depth {
        stats.max_depth = max_depth;
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

    let mut value = i32::MIN + 1;

    for m in movements {
        stats.explored += 1;
        board.do_movement(&m);
        let score = -minimax(
            stats,
            ctx,
            max_depth,
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

const MAX_DEPTH: u32 = 20;
const MAX_TIME_MS: u128 = 50;

// The main entry point for asking the Checkers engine to select a move for a given [Player]
// within the context of a given [Board] state.
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

    if ctx.iterative {
        let timer = Instant::now();
        for d in 1..=MAX_DEPTH {
            if timer.elapsed().as_millis() > MAX_TIME_MS {
                break;
            }
            let result = minimax(
                stats,
                ctx,
                0,
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
    } else {
        let result = minimax(
            stats,
            ctx,
            0,
            board,
            player,
            table,
            ctx.depth,
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
