use clap::{Parser, ValueEnum};
use rand::seq::SliceRandom;
// use std::{collections::HashMap, fmt};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum, Eq, Hash)]
enum Player {
    Player1,
    Player2,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Piece {
    player: Player,
    king: bool,
}

impl Piece {
    fn new(player: Player, king: bool) -> Self {
        Self { player, king }
    }

    fn player1_pawn() -> Self {
        Self::new(Player::Player1, false)
    }

    fn player1_king() -> Self {
        Self::new(Player::Player1, true)
    }

    fn player2_pawn() -> Self {
        Self::new(Player::Player2, false)
    }

    fn player2_king() -> Self {
        Self::new(Player::Player2, true)
    }

    fn movements(&self) -> &[i32] {
        if self.king {
            return &[-4, -5, 4, 5];
        }
        match self.player {
            Player::Player1 => &[4, 5],
            Player::Player2 => &[-4, -5],
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.player {
            Player::Player1 => {
                if self.king {
                    write!(f, "O")
                } else {
                    write!(f, "o")
                }
            }
            Player::Player2 => {
                if self.king {
                    write!(f, "X")
                } else {
                    write!(f, "x")
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Square {
    Invalid,
    Empty,
    Taken(Piece),
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "!"),
            Self::Empty => write!(f, " "),
            Self::Taken(piece) => write!(f, "{}", piece),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct SquareState {
    id: usize,
    piece: Option<Piece>,
}

impl SquareState {
    fn piece(id: usize, piece: Piece) -> Self {
        Self {
            id,
            piece: Some(piece),
        }
    }

    fn empty(id: usize) -> Self {
        Self { id, piece: None }
    }
}

#[derive(Debug, PartialEq)]
struct Movement {
    from: SquareState,
    to: SquareState,
    jumped: Option<SquareState>,
    next: Option<Box<Movement>>,
}

impl Movement {
    fn simple(from: SquareState, to: SquareState) -> Self {
        Self {
            from,
            to,
            jumped: None,
            next: None,
        }
    }

    fn jump(from: SquareState, to: SquareState, jumped: SquareState) -> Self {
        Self {
            from,
            to,
            jumped: Some(jumped),
            next: None,
        }
    }

    fn multi_jump(
        from: SquareState,
        to: SquareState,
        jumped: SquareState,
        next: Box<Movement>,
    ) -> Self {
        Self {
            from,
            to,
            jumped: Some(jumped),
            next: Some(next),
        }
    }
}

const VALID_SQUARES: [usize; 32] = [
    5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31, 32,
    33, 34, 35, 37, 38, 39, 40,
];
const PLAYER1_START: [usize; 12] = [5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17];
const PLAYER2_START: [usize; 12] = [28, 29, 30, 31, 32, 33, 34, 35, 37, 38, 39, 40];
const EMPTY_START: [usize; 8] = [19, 20, 21, 22, 23, 24, 25, 26];
const PLAYER1_KINGS: [usize; 4] = [37, 38, 39, 40];
const PLAYER2_KINGS: [usize; 4] = [5, 6, 7, 8];

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Board {
    squares: [Square; 46],
}

impl Board {
    fn new() -> Self {
        let mut squares = [Square::Invalid; 46];
        for id in PLAYER1_START {
            squares[id] = Square::Taken(Piece::player1_pawn());
        }
        for id in EMPTY_START {
            squares[id] = Square::Empty;
        }
        for id in PLAYER2_START {
            squares[id] = Square::Taken(Piece::player2_pawn());
        }
        Self { squares }
    }

    #[allow(dead_code)]
    fn empty() -> Self {
        let mut squares = [Square::Invalid; 46];
        for id in VALID_SQUARES {
            squares[id] = Square::Empty;
        }
        Self { squares }
    }

    fn get(&self, id: usize) -> Square {
        self.squares[id]
    }

    #[allow(dead_code)]
    fn set(&mut self, id: usize, square: Square) {
        self.squares[id] = square;
    }

    fn movements(&self, player: Player) -> Vec<Movement> {
        let jumps = self.jump_moves(player);
        if !jumps.is_empty() {
            return jumps;
        }
        self.simple_moves(player)
    }

    fn simple_moves(&self, player: Player) -> Vec<Movement> {
        let mut movements = Vec::new();
        for id in VALID_SQUARES {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == player {
                    for m in piece.movements() {
                        let id_to = (id as i32 + m) as usize;
                        if Square::Empty == self.squares[id_to] {
                            let from = SquareState::piece(id, piece);
                            let to = SquareState::empty(id_to);
                            let movement = Movement::simple(from, to);
                            movements.push(movement);
                        }
                    }
                }
            }
        }
        movements
    }

    fn jump_moves(&self, player: Player) -> Vec<Movement> {
        let mut movements = Vec::new();
        for id in VALID_SQUARES {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == player {
                    movements.append(&mut self.jump_moves_at(
                        player,
                        piece,
                        id,
                        id,
                        &mut Vec::new(),
                    ));
                }
            }
        }
        movements
    }

    fn jump_moves_at(
        &self,
        player: Player,
        piece: Piece,
        id: usize,
        start: usize,
        visited: &mut Vec<usize>,
    ) -> Vec<Movement> {
        let mut movements = Vec::new();
        for m in piece.movements() {
            let id_jumped = (id as i32 + m) as usize;
            let id_to = (id_jumped as i32 + m) as usize;
            if visited.iter().any(|v| *v == id_to) {
                continue;
            }
            if let Square::Taken(jumped_piece) = self.squares[id_jumped] {
                if jumped_piece.player != player && Square::Empty == self.squares[id_to]
                    || id_to == start
                {
                    let from = SquareState::piece(id, piece);
                    let to = SquareState::empty(id_to);
                    let jumped = SquareState::piece(id_jumped, jumped_piece);
                    visited.push(id_to);
                    let multi_jumps = self.jump_moves_at(player, piece, id_to, start, visited);
                    visited.pop();
                    if multi_jumps.is_empty() {
                        let movement = Movement::jump(from, to, jumped);
                        movements.push(movement);
                    } else {
                        for mj in multi_jumps {
                            let movement = Movement::multi_jump(from, to, jumped, Box::new(mj));
                            movements.push(movement);
                        }
                    }
                }
            }
        }
        movements
    }

    fn do_movement(&mut self, movement: &Movement) {
        self.squares[movement.to.id] = self.squares[movement.from.id];
        self.squares[movement.from.id] = Square::Empty;
        if let Some(jumped_state) = &movement.jumped {
            self.squares[jumped_state.id] = Square::Empty;
            if let Some(next_movement) = &movement.next {
                self.do_movement(next_movement);
            }
        }
    }

    fn undo_movement(&mut self, movement: &Movement) {
        if let Some(next_movement) = &movement.next {
            self.undo_movement(next_movement);
        }
        self.squares[movement.from.id] = self.squares[movement.to.id];
        self.squares[movement.to.id] = Square::Empty;
        if let Some(jumped_state) = &movement.jumped {
            self.squares[jumped_state.id] = Square::Taken(jumped_state.piece.unwrap());
        }
    }

    #[allow(dead_code)]
    fn piece_count(&self) -> (u8, u8) {
        let mut p1 = 0;
        let mut p2 = 0;
        for id in VALID_SQUARES {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == Player::Player1 {
                    p1 += 1;
                } else {
                    p2 += 1;
                }
            }
        }
        (p1, p2)
    }

    fn mark_kings(&mut self) {
        for id in PLAYER1_KINGS {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == Player::Player1 && !piece.king {
                    self.squares[id] = Square::Taken(Piece::player1_king());
                }
            }
        }
        for id in PLAYER2_KINGS {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == Player::Player2 && !piece.king {
                    self.squares[id] = Square::Taken(Piece::player2_king());
                }
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "1  |   | {} |   | {} |   | {} |   | {} |",
            self.squares[37], self.squares[38], self.squares[39], self.squares[40]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "2  | {} |   | {} |   | {} |   | {} |   |",
            self.squares[32], self.squares[33], self.squares[34], self.squares[35]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "3  |   | {} |   | {} |   | {} |   | {} |",
            self.squares[28], self.squares[29], self.squares[30], self.squares[31]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "4  | {} |   | {} |   | {} |   | {} |   |",
            self.squares[23], self.squares[24], self.squares[25], self.squares[26]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "5  |   | {} |   | {} |   | {} |   | {} |",
            self.squares[19], self.squares[20], self.squares[21], self.squares[22]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "6  | {} |   | {} |   | {} |   | {} |   |",
            self.squares[14], self.squares[15], self.squares[16], self.squares[17]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "7  |   | {} |   | {} |   | {} |   | {} |",
            self.squares[10], self.squares[11], self.squares[12], self.squares[13]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(
            f,
            "8  | {} |   | {} |   | {} |   | {} |   |",
            self.squares[5], self.squares[6], self.squares[7], self.squares[8]
        )?;
        writeln!(f, "   ---------------------------------")?;
        writeln!(f, "     A   B   C   D   E   F   G   H")
    }
}

// # https://3dkingdoms.com/checkers/bitboards.htm
// #
// #    37  38  39  40
// #  32  33  34  35
// #    28  29  30  31
// #  23  24  25  26
// #    19  20  21  22
// #  14  15  16  17
// #    10  11  12  13
// #  05  06  07  08
const BACK_ROW: [usize; 8] = [5, 6, 7, 8, 37, 38, 39, 40];
fn evaluate(player: Player, board: &Board) -> i32 {
    let mut pawns = 0;
    let mut kings = 0;
    let mut back_row = 0;
    let mut total = 0;
    for id in VALID_SQUARES {
        if let Square::Taken(piece) = board.get(id) {
            total += 1;
            if piece.player == player {
                if piece.king {
                    kings += 1;
                } else {
                    pawns += 1;
                }
            } else if piece.king {
                kings -= 1;
            } else {
                pawns -= 1;
            }
        }
    }
    if total > 16 {
        for id in BACK_ROW {
            if let Square::Taken(piece) = board.get(id) {
                if piece.player == player && !piece.king {
                    back_row += 1;
                } else if !piece.king {
                    back_row -= 1;
                }
            }
        }
    }
    (2 * pawns) + (5 * kings) + back_row
}

const MAX: i32 = i32::MAX - 1;
const MIN: i32 = i32::MIN + 1;

// "Artificial Intelligence: A Modern Approach"
// -- 5.2.1 The minimax algorithm
fn minimax(
    player: Player,
    board: &mut Board,
    depth: u8,
    maximizing: bool,
    explored: &mut u32,
) -> i32 {
    if depth == 0 {
        let maximizing_player = if maximizing { player } else { player.other() };
        return evaluate(maximizing_player, board);
    }
    if maximizing {
        let mut value = MIN;
        let movements = board.movements(player);
        for m in movements {
            *explored += 1;
            board.do_movement(&m);
            value = value.max(minimax(player.other(), board, depth - 1, false, explored));
            board.undo_movement(&m);
        }
        value
    } else {
        let mut value = MAX;
        let movements = board.movements(player);
        for m in movements {
            *explored += 1;
            board.do_movement(&m);
            value = value.min(minimax(player.other(), board, depth - 1, true, explored));
            board.undo_movement(&m);
        }
        value
    }
}

// "Artificial Intelligence: A Modern Approach"
// -- 5.3 Alpha-Beta Pruning
fn alphabeta(
    player: Player,
    board: &mut Board,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    explored: &mut u32,
) -> i32 {
    if depth == 0 {
        let maximizing_player = if maximizing { player } else { player.other() };
        return evaluate(maximizing_player, board);
    }
    if maximizing {
        let mut value = MIN;
        let movements = board.movements(player);
        for m in movements {
            *explored += 1;
            board.do_movement(&m);
            value = value.max(alphabeta(
                player.other(),
                board,
                depth - 1,
                alpha,
                beta,
                false,
                explored,
            ));
            board.undo_movement(&m);
            if value >= beta {
                break;
            }
            alpha = alpha.max(value);
        }
        value
    } else {
        let mut value = MAX;
        let movements = board.movements(player);
        for m in movements {
            *explored += 1;
            board.do_movement(&m);
            value = value.min(alphabeta(
                player.other(),
                board,
                depth - 1,
                alpha,
                beta,
                true,
                explored,
            ));
            board.undo_movement(&m);
            if value <= alpha {
                break;
            }
            beta = beta.min(value);
        }
        value
    }
}

fn minimax_root(
    player: Player,
    board: &mut Board,
    alpha_beta: bool,
    explored: &mut u32,
) -> Option<Movement> {
    let movements = board.movements(player);

    if movements.is_empty() {
        return None;
    }

    let mut value = MIN - 1;
    let mut movement = None;

    for m in movements {
        board.do_movement(&m);
        let v = if alpha_beta {
            alphabeta(player.other(), board, 6, MIN, MAX, false, explored)
        } else {
            minimax(player.other(), board, 6, false, explored)
        };
        board.undo_movement(&m);
        if v > value {
            movement = Some(m);
            value = v;
        }
    }

    movement
}

// #[derive(Debug)]
// enum Flag {
//     Exact,
//     Lowerbound,
//     Upperbound,
// }

// #[derive(Debug)]
// struct Entry {
//     value: i32,
//     depth: u8,
//     flag: Flag,
// }

// #[derive(Debug)]
// struct Table {
//     entries: HashMap<Board, Entry>,
// }

// impl Table {
//     fn new() -> Self {
//         Self {
//             entries: HashMap::new(),
//         }
//     }

//     fn get(&self, board: &Board) -> Option<&Entry> {
//         self.entries.get(board)
//     }

//     fn insert(&mut self, board: &Board, value: i32, flag: Flag, depth: u8) {
//         if let Some(entry) = self.entries.get_mut(board) {
//             entry.value = value;
//             entry.flag = flag;
//             entry.depth = depth;
//         } else {
//             self.entries.insert(*board, Entry { value, flag, depth });
//         }
//     }
// }

// #[derive(Debug)]
// struct Context {
//     alpha_beta: bool,
//     transposition_table: Option<Table>,
//     depth: u8,
//     explored: u32,
//     table_hits: u32,
// }

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    alpha_beta: bool,
    #[arg(short, long)]
    transposition_table: bool,
    #[arg(short, long, default_value_t = 1)]
    games: u32,
    #[arg(short, long, default_value_t = 7)]
    depth: u8,
}

fn main() {
    let cli = Cli::parse();

    // let mut context = Context {
    //     alpha_beta: cli.alpha_beta,
    //     transposition_table: None,
    //     depth: cli.depth,
    //     explored: 0,
    //     table_hits: 0,
    // };

    // if cli.transposition_table {
    //     context.transposition_table = Some(Table::new());
    // }

    let mut player1 = 0;
    let mut player2 = 0;
    let mut explored = 0;

    for _ in 0..cli.games {
        let mut board = Board::new();
        let loser;
        loop {
            if let Some(movement) =
                minimax_root(Player::Player1, &mut board, cli.alpha_beta, &mut explored)
            {
                board.do_movement(&movement);
            } else {
                loser = Player::Player1;
                break;
            }
            board.mark_kings();

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
        println!("{}", explored);
        explored = 0;
    }
    dbg!(player1);
    dbg!(player2);
}

#[cfg(test)]
mod test {
    use super::*;

    // # https://3dkingdoms.com/checkers/bitboards.htm
    // #
    // #    37  38  39  40
    // #  32  33  34  35
    // #    28  29  30  31
    // #  23  24  25  26
    // #    19  20  21  22
    // #  14  15  16  17
    // #    10  11  12  13
    // #  05  06  07  08

    #[test]
    fn test_simple_movements() {
        let board_new = Board::new();
        let mut board = Board::new();
        let movement = Movement::simple(
            SquareState::piece(15, Piece::player1_pawn()),
            SquareState::empty(19),
        );
        assert!(board
            .simple_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        board.do_movement(&movement);
        assert_ne!(board_new, board);
        board.undo_movement(&movement);
        assert_eq!(board_new, board);
    }

    #[test]
    fn test_do_movement_jump() {
        let mut board = Board::new();
        let m1 = Movement::simple(
            SquareState::piece(15, Piece::player1_pawn()),
            SquareState::empty(20),
        );
        let m2 = Movement::simple(
            SquareState::piece(30, Piece::player2_pawn()),
            SquareState::empty(25),
        );
        board.do_movement(&m1);
        board.do_movement(&m2);
        let movement = Movement::jump(
            SquareState::piece(20, Piece::player1_pawn()),
            SquareState::empty(30),
            SquareState::piece(25, Piece::player2_pawn()),
        );
        assert!(!board
            .simple_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        assert!(board
            .jump_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        board.do_movement(&movement);
        assert_eq!(board.get(25), Square::Empty);
        board.undo_movement(&movement);
        board.undo_movement(&m2);
        board.undo_movement(&m1);
        assert_eq!(board, Board::new());
    }

    #[test]
    fn test_do_movement_multi_jump() {
        let mut board = Board::new();
        let m1 = Movement::simple(
            SquareState::piece(15, Piece::player1_pawn()),
            SquareState::empty(20),
        );
        let m2 = Movement::simple(
            SquareState::piece(29, Piece::player2_pawn()),
            SquareState::empty(24),
        );
        let m3 = Movement::simple(
            SquareState::piece(11, Piece::player1_pawn()),
            SquareState::empty(15),
        );
        let m4 = Movement::simple(
            SquareState::piece(33, Piece::player2_pawn()),
            SquareState::empty(29),
        );
        let m5 = Movement::simple(
            SquareState::piece(17, Piece::player1_pawn()),
            SquareState::empty(22),
        );
        let m6 = Movement::simple(
            SquareState::piece(38, Piece::player2_pawn()),
            SquareState::empty(33),
        );
        let m7 = Movement::simple(
            SquareState::piece(6, Piece::player1_pawn()),
            SquareState::empty(11),
        );
        let m8 = Movement::simple(
            SquareState::piece(30, Piece::player2_pawn()),
            SquareState::empty(25),
        );
        let movement = Movement::multi_jump(
            SquareState::piece(20, Piece::player1_pawn()),
            SquareState::empty(30),
            SquareState::piece(25, Piece::player2_pawn()),
            Box::new(Movement::jump(
                SquareState::piece(30, Piece::player1_pawn()),
                SquareState::empty(38),
                SquareState::piece(34, Piece::player2_pawn()),
            )),
        );
        board.do_movement(&m1);
        board.do_movement(&m2);
        board.do_movement(&m3);
        board.do_movement(&m4);
        board.do_movement(&m5);
        board.do_movement(&m6);
        board.do_movement(&m7);
        board.do_movement(&m8);
        assert!(!board
            .simple_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        assert!(board
            .jump_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        board.do_movement(&movement);
        let (p1, p2) = board.piece_count();
        assert_eq!(p1, 12);
        assert_eq!(p2, 10);
        board.undo_movement(&movement);
        board.undo_movement(&m8);
        board.undo_movement(&m7);
        board.undo_movement(&m6);
        board.undo_movement(&m5);
        board.undo_movement(&m4);
        board.undo_movement(&m3);
        board.undo_movement(&m2);
        board.undo_movement(&m1);
        assert_eq!(board, Board::new());
    }

    #[test]
    fn test_king_circle_jump() {
        let mut board = Board::empty();
        board.set(11, Square::Taken(Piece::player1_king()));
        board.set(16, Square::Taken(Piece::player2_pawn()));
        board.set(25, Square::Taken(Piece::player2_pawn()));
        board.set(24, Square::Taken(Piece::player2_pawn()));
        board.set(15, Square::Taken(Piece::player2_pawn()));
        let jumps = board.jump_moves(Player::Player1);
        let movement = Movement::multi_jump(
            SquareState::piece(11, Piece::player1_king()),
            SquareState::empty(21),
            SquareState::piece(16, Piece::player2_pawn()),
            Box::new(Movement::multi_jump(
                SquareState::piece(21, Piece::player1_king()),
                SquareState::empty(29),
                SquareState::piece(25, Piece::player2_pawn()),
                Box::new(Movement::multi_jump(
                    SquareState::piece(29, Piece::player1_king()),
                    SquareState::empty(19),
                    SquareState::piece(24, Piece::player2_pawn()),
                    Box::new(Movement::jump(
                        SquareState::piece(19, Piece::player1_king()),
                        SquareState::empty(11),
                        SquareState::piece(15, Piece::player2_pawn()),
                    )),
                )),
            )),
        );
        assert!(jumps.iter().any(|m| *m == movement));
        board.do_movement(&movement);
        assert_eq!(board.get(16), Square::Empty);
        assert_eq!(board.get(25), Square::Empty);
        assert_eq!(board.get(24), Square::Empty);
        assert_eq!(board.get(15), Square::Empty);
        board.undo_movement(&movement);
        assert_eq!(board.get(16), Square::Taken(Piece::player2_pawn()));
        assert_eq!(board.get(25), Square::Taken(Piece::player2_pawn()));
        assert_eq!(board.get(24), Square::Taken(Piece::player2_pawn()));
        assert_eq!(board.get(15), Square::Taken(Piece::player2_pawn()));
    }

    #[test]
    fn test_negamax_is_same_as_alpha_beta_and_table() {
        let mut board1 = Board::new();
        let mut move_list_1 = Vec::new();
        loop {
            if let Some(movement) = minimax_root(Player::Player1, &mut board1, true, &mut 0) {
                board1.do_movement(&movement);
                move_list_1.push(movement);
            } else {
                break;
            }
            board1.mark_kings();
            if let Some(movement) = minimax_root(Player::Player2, &mut board1, false, &mut 0) {
                board1.do_movement(&movement);
                move_list_1.push(movement);
            } else {
                break;
            }
            board1.mark_kings();
        }

        let mut board2 = Board::new();
        let mut move_list_2 = Vec::new();
        loop {
            if let Some(movement) = minimax_root(Player::Player1, &mut board2, false, &mut 0) {
                board2.do_movement(&movement);
                move_list_2.push(movement);
            } else {
                break;
            }
            board2.mark_kings();
            if let Some(movement) = minimax_root(Player::Player2, &mut board2, true, &mut 0) {
                board2.do_movement(&movement);
                move_list_2.push(movement);
            } else {
                break;
            }
            board2.mark_kings();
        }

        assert_eq!(move_list_1, move_list_2);
    }
}
