use std::collections::HashMap;

use rand::prelude::SliceRandom;

use crate::{
    checkers::{Board, Movement, Player},
    human::{get_user_input, MovementMap},
    minimax::{get_movement, MinimaxContext, Stats, TTEntry},
};

enum RunnerKind {
    Random,
    AI,
    Human,
}

pub struct Runner {
    kind: RunnerKind,
    context: Option<MinimaxContext>,
    table: Option<HashMap<Board, TTEntry>>,
    map: Option<MovementMap>,
    stats: Stats,
}

impl Runner {
    pub fn random() -> Self {
        Self {
            kind: RunnerKind::Random,
            context: None,
            table: None,
            map: None,
            stats: Stats::new(),
        }
    }

    pub fn ai(context: MinimaxContext, table: HashMap<Board, TTEntry>) -> Self {
        Self {
            kind: RunnerKind::AI,
            context: Some(context),
            table: Some(table),
            map: None,
            stats: Stats::new(),
        }
    }

    pub fn human(map: MovementMap) -> Self {
        Self {
            kind: RunnerKind::Human,
            context: None,
            table: None,
            map: Some(map),
            stats: Stats::new(),
        }
    }

    pub fn display_stats(&self) {
        println!("moves = {}", self.stats.moves);
        println!("explored = {}", self.stats.explored);
        println!("beta_cuts = {}", self.stats.beta_cuts);
        println!("tt_exact = {}", self.stats.tt_exact);
        println!("tt_cuts = {}", self.stats.tt_cuts);
    }

    pub fn get_move(&mut self, board: &mut Board, player: Player) -> Option<Movement> {
        match self.kind {
            RunnerKind::Random => {
                let movements = board.movements(player);
                if movements.is_empty() {
                    return None;
                }
                self.stats.moves += 1;
                movements.choose(&mut rand::thread_rng()).cloned()
            }
            RunnerKind::AI => get_movement(
                &mut self.stats,
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
                            self.stats.moves += 1;
                            return Some(movement);
                        }
                    }
                }
            }
        }
    }
}
