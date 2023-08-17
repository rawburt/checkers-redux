use clap::ValueEnum;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Square {
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
pub struct SquareState {
    id: usize,
    piece: Option<Piece>,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Movement {
    from: SquareState,
    to: SquareState,
    jumped: Option<SquareState>,
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
    // "Some Studies in Machine Learning Using the Game of Checkers" by Arthur L. Samuel
    //      -- alternative board layout, similar approach as Jonathan Kreuzer above
    //
    squares: [Square; 46],
}

impl Board {
    pub fn new() -> Self {
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
    pub fn empty() -> Self {
        let mut squares = [Square::Invalid; 46];
        for id in VALID_SQUARES {
            squares[id] = Square::Empty;
        }
        Self { squares }
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

    pub fn do_movement(&mut self, movement: &Movement) {
        self.squares[movement.to.id] = self.squares[movement.from.id];
        self.squares[movement.from.id] = Square::Empty;
        if let Some(jumped_state) = &movement.jumped {
            self.squares[jumped_state.id] = Square::Empty;
            if let Some(next_movement) = &movement.next {
                self.do_movement(next_movement);
            }
        }
    }

    pub fn undo_movement(&mut self, movement: &Movement) {
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
}
