use std::time::{Duration, Instant};

use chess::{BoardStatus, ChessMove, MoveGen};
use log::log;

use crate::engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo};

pub struct Negamax {
    nodes_explored: usize,
}

impl<E: EvaluateEngine> SearchEngine<E> for Negamax {
    fn next_move(
        &mut self,
        mut state: GameState,
        time_info: &Option<TimeInfo>,
    ) -> Option<ChessMove> {
        self.nodes_explored = 0;
        let start = Instant::now();

        let board = state.last_board();
        let mut best_score = i16::MIN;
        let mut best_move = None;

        let legal_moves = MoveGen::new_legal(&board);

        for mv in legal_moves {
            state.make_move(mv);
            let score = self.search_eval::<E>(&mut state, time_info, 2);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            log::info!(
                "Nodes explored: {} in {}ms",
                self.nodes_explored,
                (Instant::now() - start).as_millis()
            );
            state.undo_last_move();
        }
        log::info!(
            "Nodes explored: {} in {}ms",
            self.nodes_explored,
            (Instant::now() - start).as_millis()
        );
        best_move
    }
}

impl Negamax {
    pub fn new() -> Self {
        Self { nodes_explored: 0 }
    }
    pub fn search_eval<E: EvaluateEngine>(
        &mut self,
        state: &mut GameState,
        time_info: &Option<TimeInfo>,
        depth: u16,
    ) -> i16 {
        self.nodes_explored += 1;
        if depth == 0 {
            return E::evaluate(state);
        }
        let board = state.last_board();

        match board.status() {
            BoardStatus::Ongoing => MoveGen::new_legal(&board)
                .map(|mv| {
                    state.make_move(mv);
                    let score = self.search_eval::<E>(state, time_info, depth - 1);
                    state.undo_last_move();
                    score
                })
                .max()
                .unwrap(),
            BoardStatus::Stalemate => return 0,
            BoardStatus::Checkmate => return i16::MIN
            
        }
    }
}
