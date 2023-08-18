use std::collections::HashMap;
use std::io::Write;

use crate::checkers::{Board, Movement, Player, Square, SquareState};

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

fn parse_jump(
    board: &Board,
    map: &MovementMap,
    steps: &[&str],
    idx: usize,
    moving: Option<&SquareState>,
) -> Option<Movement> {
    if steps.len() <= idx + 2 {
        return None;
    }
    let start = map.get(steps[idx])?;
    let jumped = map.get(steps[idx + 1])?;
    let end = map.get(steps[idx + 2])?;

    // nested jump from a multi-jump
    if let Some(m) = moving {
        if let Square::Taken(jumped_piece) = board.get(*jumped) {
            let square_start = SquareState::piece(*start, m.piece.unwrap());
            let square_jumped = SquareState::piece(*jumped, jumped_piece);
            let square_end = SquareState::empty(*end);
            return Some(Movement::jump(square_start, square_end, square_jumped));
        }
    }

    // normal jump or start of multi-jump
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

fn parse_multi_jump(
    board: &Board,
    map: &MovementMap,
    steps: &Vec<&str>,
    idx: usize,
    parent: &mut Movement,
    moving: SquareState,
) {
    if steps.len() <= idx {
        return;
    }
    if steps[idx] != "J:" {
        panic!("expected jump 1");
    }
    match parse_jump(board, map, steps, idx + 1, Some(&moving)) {
        None => panic!("expected jump 2"),
        Some(mut m) => {
            parse_multi_jump(board, map, steps, idx + 4, &mut m, moving);
            parent.set_next(&m);
        }
    }
}

pub fn parse_input(line: &mut str, board: &Board, map: &MovementMap) -> Option<Movement> {
    let steps: Vec<&str> = line.trim().split(' ').collect();

    if steps.len() < 3 {
        if !steps.is_empty() && steps[0] == "?" {
            dbg!(board.movements(Player::Player1));
        }
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
        "J:" => parse_jump(board, map, &steps, 1, None),
        "M:" => {
            let mut jump = parse_jump(board, map, &steps, 2, None)?;
            let moving = jump.from();
            parse_multi_jump(board, map, &steps, 5, &mut jump, moving);
            Some(jump)
        }
        _ => None,
    }
}

pub fn get_user_input(board: &Board, map: &MovementMap) -> Option<Movement> {
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    parse_input(&mut line, board, map)
}

#[cfg(test)]
mod test {
    use crate::checkers::Piece;

    use super::*;

    #[test]
    fn test_parse_multi_jump() {
        let mut board = Board::empty();
        board.set(10, Square::Taken(Piece::player1_pawn()));
        board.set(15, Square::Taken(Piece::player2_pawn()));
        board.set(25, Square::Taken(Piece::player2_pawn()));
        let map = MovementMap::new();
        let mut input = "M: J: B7 C6 D5 J: D5 E4 F3".to_string();
        let movement = parse_input(&mut input, &board, &map);
        assert!(movement.is_some());
        let expected = Movement::multi_jump(
            SquareState::piece(10, Piece::player1_pawn()),
            SquareState::empty(20),
            SquareState::piece(15, Piece::player2_pawn()),
            Box::new(Movement::jump(
                SquareState::piece(20, Piece::player1_pawn()),
                SquareState::empty(30),
                SquareState::piece(25, Piece::player2_pawn()),
            )),
        );
        assert_eq!(expected, movement.unwrap());
    }

    #[test]
    fn test_parse_jump() {
        let mut board = Board::empty();
        board.set(17, Square::Taken(Piece::player1_pawn()));
        board.set(21, Square::Taken(Piece::player2_pawn()));
        let map = MovementMap::new();
        let mut input = "J: G6 F5 E4".to_string();
        let movement = parse_input(&mut input, &board, &map);
        assert!(movement.is_some());
        let expected = Movement::jump(
            SquareState::piece(17, Piece::player1_pawn()),
            SquareState::empty(25),
            SquareState::piece(21, Piece::player2_pawn()),
        );
        assert_eq!(expected, movement.unwrap());
    }
}
