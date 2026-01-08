use std::time::{Duration, Instant};

use chess::{BoardStatus, ChessMove, MoveGen};

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
            state.undo_last_move();
        }
        let elapsed = (Instant::now() - start);
        log::info!(
            "Nodes explored: {} in {}ms. {:.0} NPS",
            self.nodes_explored,
            elapsed.as_millis(),
            (self.nodes_explored as f64 / elapsed.as_secs_f64()).round()
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
            BoardStatus::Checkmate => {
                return -crate::evaluation::MATE_VALUE + state.num_moves as i16;
            }
        }
    }
}
