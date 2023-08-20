// This module contains the main data structures that represent board state in the Checkers engine.

use clap::ValueEnum;
use rand::{thread_rng, Rng};
use std::fmt;

// Define the two players of a Checkers game.
#[derive(Debug, PartialEq, Clone, Copy, ValueEnum, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    // Returns the opposite player.
    pub fn other(&self) -> Player {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

// Define the types of pieces in a Checkers game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    // What player the piece belongs to.
    player: Player,
    // If the piece is a king.
    king: bool,
}

impl Piece {
    fn new(player: Player, king: bool) -> Self {
        Self { player, king }
    }

    pub fn player1_pawn() -> Self {
        Self::new(Player::Player1, false)
    }

    pub fn player1_king() -> Self {
        Self::new(Player::Player1, true)
    }

    pub fn player2_pawn() -> Self {
        Self::new(Player::Player2, false)
    }

    pub fn player2_king() -> Self {
        Self::new(Player::Player2, true)
    }

    pub fn id(&self) -> usize {
        ZobristHash::piece_id(*self)
    }

    // Return a slice of possible moves given the current state of the piece.
    pub fn movements(&self) -> &[i32] {
        if self.king {
            return &[-4, -5, 4, 5];
        }
        match self.player {
            Player::Player1 => &[4, 5],
            Player::Player2 => &[-4, -5],
        }
    }

    pub fn get_player(&self) -> Player {
        self.player
    }

    pub fn is_king(&self) -> bool {
        self.king
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

// Define the state of any square on the [Board].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Square {
    // Invalid squares are not playable. This is a result of the padded array
    // data structure used in the [Board] definition.
    Invalid,
    // Empty squares are playable.
    Empty,
    // Taken squares have a piece currently occupying them.
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

// [SquareState] is used in [Movement] to represent a location on the [Board] and what
// piece is there are the time of constructing a [Movement]. The piece state is saved
// in order to undo movements.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SquareState {
    // The location on the [Board].
    pub id: usize,
    // The piece state occupying the location.
    pub piece: Option<Piece>,
}

impl SquareState {
    pub fn piece(id: usize, piece: Piece) -> Self {
        Self {
            id,
            piece: Some(piece),
        }
    }

    pub fn empty(id: usize) -> Self {
        Self { id, piece: None }
    }
}

// Define the information required to move a piece on the board.
#[derive(Debug, PartialEq, Clone)]
pub struct Movement {
    // From which square the piece is moving.
    from: SquareState,
    // To which square the piece is moving.
    to: SquareState,
    // The piece that was jumped (if any).
    jumped: Option<SquareState>,
    // The next jump in the movement sequence (if any).
    next: Option<Box<Movement>>,
}

impl Movement {
    pub fn simple(from: SquareState, to: SquareState) -> Self {
        Self {
            from,
            to,
            jumped: None,
            next: None,
        }
    }

    pub fn jump(from: SquareState, to: SquareState, jumped: SquareState) -> Self {
        Self {
            from,
            to,
            jumped: Some(jumped),
            next: None,
        }
    }

    pub fn multi_jump(
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

    pub fn set_next(&mut self, movement: &Movement) {
        self.next = Some(Box::new(movement.clone()));
    }

    pub fn is_jump(&self) -> bool {
        self.jumped.is_some()
    }

    pub fn from(&self) -> SquareState {
        self.from
    }
}

// Define the Zobrist hash data structure for a [Board].
#[derive(Debug, PartialEq, Clone, Copy)]
struct ZobristHash {
    // Each board piece may occupy 4 different states:
    //      * Player 1 pawn
    //      * Player 1 king
    //      * Player 2 pawn
    //      * Player 2 king
    // The board is a 46 element padded array. Thus, we use
    // a 46 element array of 4 element array u128 random numbers.
    randoms: [[u128; 4]; 46],
    // The currenty hash of the board that the [ZobristHash] is
    // hashing.
    hash: u128,
}

impl ZobristHash {
    fn new() -> Self {
        let mut randoms = [[0; 4]; 46];
        for r in &mut randoms {
            r[0] = thread_rng().gen();
            r[1] = thread_rng().gen();
            r[2] = thread_rng().gen();
            r[3] = thread_rng().gen();
        }
        Self { randoms, hash: 0 }
    }

    fn piece_id(piece: Piece) -> usize {
        match piece.player {
            Player::Player1 => {
                if piece.king {
                    1
                } else {
                    0
                }
            }
            Player::Player2 => {
                if piece.king {
                    3
                } else {
                    2
                }
            }
        }
    }

    fn flip(&mut self, pos: usize, piece: usize) {
        self.hash ^= self.randoms[pos][piece];
    }
}

pub const VALID_SQUARES: [usize; 32] = [
    5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31, 32,
    33, 34, 35, 37, 38, 39, 40,
];
const PLAYER1_START: [usize; 12] = [5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17];
const PLAYER2_START: [usize; 12] = [28, 29, 30, 31, 32, 33, 34, 35, 37, 38, 39, 40];
const EMPTY_START: [usize; 8] = [19, 20, 21, 22, 23, 24, 25, 26];
const PLAYER1_KINGS: [usize; 4] = [37, 38, 39, 40];
const PLAYER2_KINGS: [usize; 4] = [5, 6, 7, 8];

#[derive(Debug)]
pub struct Board {
    // # https://3dkingdoms.com/checkers/bitboards.htm by Jonathan Kreuzer
    // #
    // #    37  38  39  40      |- Player 2 start (X)
    // #  32  33  34  35        |-
    // #    28  29  30  31      |-
    // #  23  24  25  26
    // #    19  20  21  22
    // #  14  15  16  17        |-
    // #    10  11  12  13      |-
    // #  05  06  07  08        |- Player 1 start (O)
    //
    squares: [Square; 46],
    // The current Zobrist hash of the board state.
    zobrist: ZobristHash,
}

impl Board {
    pub fn new() -> Self {
        let mut zobrist = ZobristHash::new();
        let mut squares = [Square::Invalid; 46];
        for id in PLAYER1_START {
            let p = Piece::player1_pawn();
            squares[id] = Square::Taken(p);
            zobrist.flip(id, p.id())
        }
        for id in EMPTY_START {
            squares[id] = Square::Empty;
        }
        for id in PLAYER2_START {
            let p = Piece::player2_pawn();
            squares[id] = Square::Taken(p);
            zobrist.flip(id, p.id())
        }
        Self { squares, zobrist }
    }

    pub fn hash(&self) -> u128 {
        self.zobrist.hash
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        let zobrist = ZobristHash::new();
        let mut squares = [Square::Invalid; 46];
        for id in VALID_SQUARES {
            squares[id] = Square::Empty;
        }
        Self { squares, zobrist }
    }

    pub fn get(&self, id: usize) -> Square {
        self.squares[id]
    }

    #[allow(dead_code)]
    pub fn set(&mut self, id: usize, square: Square) {
        self.squares[id] = square;
    }

    pub fn movements(&self, player: Player) -> Vec<Movement> {
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
        prev_jumped: &mut Vec<usize>,
    ) -> Vec<Movement> {
        let mut movements = Vec::new();
        for m in piece.movements() {
            let id_jumped = (id as i32 + m) as usize;
            let id_to = (id_jumped as i32 + m) as usize;
            if prev_jumped.iter().any(|j| *j == id_jumped) {
                continue;
            }
            if let Square::Taken(jumped_piece) = self.squares[id_jumped] {
                if jumped_piece.player != player && Square::Empty == self.squares[id_to]
                    || id_to == start
                {
                    let from = SquareState::piece(id, piece);
                    let to = SquareState::empty(id_to);
                    let jumped = SquareState::piece(id_jumped, jumped_piece);
                    prev_jumped.push(id_jumped);
                    let multi_jumps = self.jump_moves_at(player, piece, id_to, start, prev_jumped);
                    prev_jumped.pop();
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

    // Change the board state based on the given [Movement]. Updates the [ZobristHash].
    pub fn do_movement(&mut self, movement: &Movement) {
        self.squares[movement.to.id] = self.squares[movement.from.id];
        self.zobrist
            .flip(movement.to.id, movement.from.piece.unwrap().id());
        self.squares[movement.from.id] = Square::Empty;
        self.zobrist
            .flip(movement.from.id, movement.from.piece.unwrap().id());
        if let Some(jumped_state) = &movement.jumped {
            self.squares[jumped_state.id] = Square::Empty;
            self.zobrist
                .flip(jumped_state.id, jumped_state.piece.unwrap().id());
            if let Some(next_movement) = &movement.next {
                self.do_movement(next_movement);
            }
        }
    }

    // Undo the board state based on the given [Movement]. Updates the [ZobristHash].
    pub fn undo_movement(&mut self, movement: &Movement) {
        if let Some(next_movement) = &movement.next {
            self.undo_movement(next_movement);
        }
        self.squares[movement.from.id] = self.squares[movement.to.id];
        self.zobrist
            .flip(movement.from.id, movement.from.piece.unwrap().id());
        self.squares[movement.to.id] = Square::Empty;
        self.zobrist
            .flip(movement.to.id, movement.from.piece.unwrap().id());
        if let Some(jumped_state) = &movement.jumped {
            self.squares[jumped_state.id] = Square::Taken(jumped_state.piece.unwrap());
            self.zobrist
                .flip(jumped_state.id, jumped_state.piece.unwrap().id());
        }
    }

    #[allow(dead_code)]
    pub fn piece_count(&self) -> (u8, u8) {
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

    pub fn mark_kings(&mut self) -> u32 {
        let mut kings = 0;
        for id in PLAYER1_KINGS {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == Player::Player1 && !piece.king {
                    self.squares[id] = Square::Taken(Piece::player1_king());
                    kings += 1;
                }
            }
        }
        for id in PLAYER2_KINGS {
            if let Square::Taken(piece) = self.squares[id] {
                if piece.player == Player::Player2 && !piece.king {
                    self.squares[id] = Square::Taken(Piece::player2_king());
                    kings += 1;
                }
            }
        }
        kings
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::new()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_movements() {
        let board_new = Board::new();
        let mut board = Board::new();
        let hash = board.hash();
        let movement = Movement::simple(
            SquareState::piece(15, Piece::player1_pawn()),
            SquareState::empty(19),
        );
        assert!(board
            .simple_moves(Player::Player1)
            .iter()
            .any(|m| *m == movement));
        board.do_movement(&movement);
        assert_ne!(board_new.squares, board.squares);
        board.undo_movement(&movement);
        assert_eq!(board_new.squares, board.squares);
        assert_eq!(hash, board.hash());
    }

    #[test]
    fn test_do_movement_jump() {
        let mut board = Board::new();
        let hash = board.hash();
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
        assert_eq!(board.squares, Board::new().squares);
        assert_eq!(hash, board.hash());
    }

    #[test]
    fn test_do_movement_multi_jump() {
        let mut board = Board::new();
        let hash = board.hash();
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
        assert_eq!(board.squares, Board::new().squares);
        assert_eq!(hash, board.hash());
    }

    #[test]
    fn test_king_circle_jump() {
        let mut board = Board::empty();
        let hash = board.hash();
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
        assert_eq!(hash, board.hash());
    }

    #[test]
    fn test_king_jump() {
        let mut board = Board::empty();
        board.set(11, Square::Taken(Piece::player2_king()));
        board.set(16, Square::Taken(Piece::player1_pawn()));
        let jumps = board.jump_moves(Player::Player2);
        let movement = Movement::jump(
            SquareState::piece(11, Piece::player2_king()),
            SquareState::empty(21),
            SquareState::piece(16, Piece::player1_pawn()),
        );
        assert!(jumps.iter().any(|m| *m == movement));
        board.do_movement(&movement);
        assert_eq!(board.get(11), Square::Empty);
        assert_eq!(board.get(16), Square::Empty);
        assert_eq!(board.get(21), Square::Taken(Piece::player2_king()));
        board.undo_movement(&movement);
        assert_eq!(board.get(11), Square::Taken(Piece::player2_king()));
        assert_eq!(board.get(16), Square::Taken(Piece::player1_pawn()));
        assert_eq!(board.get(21), Square::Empty);
    }
}
