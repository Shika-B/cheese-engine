
use chess::{ChessMove, MoveGen};

use crate::engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo};

pub struct Negamax {}

impl<E: EvaluateEngine> SearchEngine<E> for Negamax {
    fn next_move(&mut self, mut state: GameState, time_info: &Option<TimeInfo>) -> Option<ChessMove> {
        let board = state.last_board();
        let legal_moves = MoveGen::new_legal(&board);
        let mut best_score = i16::MIN;
        let mut best_move = None;

        for mv in legal_moves {
            state.make_move(mv);
            let score = self.search_eval::<E>(&mut state, time_info, 2);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            state.undo_last_move();
        }
        best_move
    }
}

impl Negamax {
    pub fn new() -> Self {
        Self {}
    }
    pub fn search_eval<E: EvaluateEngine>(
        &mut self,
        state: &mut GameState,
        time_info: &Option<TimeInfo>,
        depth: u16,
    ) -> i16 {
        if depth == 0 {
            return E::evaluate(state);
        }
        let board = state.last_board();

        MoveGen::new_legal(&board)
            .map(|mv| {
                state.make_move(mv);
                let score = self.search_eval::<E>(state, time_info, depth - 1);
                state.undo_last_move();
                score
            })
            .max()
            .unwrap()
    }
}
