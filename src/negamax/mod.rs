use std::{
    i16,
    time::{Duration, Instant},
};

use chess::{BoardStatus, ChessMove, MoveGen};

use crate::engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo};

const TRANSPOTION_TABLE_SIZE: usize = 65_536; // 65_536 = 2**16
const MAGIC_TRANSPO: usize = 48; //64 - 16

const MATE_THRESHOLD: i16 = 29_000;

#[derive(Debug, Copy, Clone)]
pub enum ResultKind {
    Exact,
    LowerBound,
    UpperBound,
    None,
}

impl Default for ResultKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct SearchResult {
    hash: u64,
    depth: u16,
    score: i16,
    kind: ResultKind,
}

pub struct Negamax {
    nodes_explored: usize,
    transposition_table: [Option<SearchResult>; TRANSPOTION_TABLE_SIZE],
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
            let score = self.search_eval::<E>(&mut state, time_info, -i16::MAX, i16::MAX, 3);
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
        Self {
            nodes_explored: 0,
            transposition_table: [None; TRANSPOTION_TABLE_SIZE],
        }
    }

    pub fn search_eval<E: EvaluateEngine>(
        &mut self,
        state: &mut GameState,
        _time_info: &Option<TimeInfo>,
        mut alpha: i16,
        beta: i16,
        depth: u16,
    ) -> i16 {
        self.nodes_explored += 1;
        let board = state.last_board();
        let board_hash = board.get_hash();
        let transpo_idx = (board_hash >> MAGIC_TRANSPO) as usize;

        if let Some(entry) = self.transposition_table[transpo_idx] {
            if entry.hash == board_hash && entry.depth >= depth {
                match entry.kind {
                    ResultKind::Exact => return entry.score,
                    ResultKind::LowerBound if entry.score >= beta => return entry.score,
                    ResultKind::UpperBound if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
        }

        if depth == 0 {
            return E::evaluate(state);
        }

        match board.status() {
            BoardStatus::Stalemate | BoardStatus::Checkmate => {
                return E::evaluate(state);
            }
            BoardStatus::Ongoing => (),
        }

        // Store so that we can decide later the kind of bound we want to store in the transposition table
        let alpha_orig = alpha.clone();

        let mut best_score = -i16::MAX;

        for mv in MoveGen::new_legal(&board) {
            state.make_move(mv);

            let score: i16 = -self.search_eval::<E>(state, _time_info, -beta, -alpha, depth - 1);

            state.undo_last_move();

            if score > best_score {
                best_score = score;
            }

            // Alpha-beta pruning
            if best_score > alpha {
                alpha = best_score;
            }
            if alpha >= beta {
                break;
            }
        }
        let kind = if best_score > MATE_THRESHOLD {
            ResultKind::None // Mate scores depend on the depths they are considered at, so we should avoid scoring them. Could probably be fixed by a better design later on.
        } else if best_score <= alpha_orig {
            ResultKind::UpperBound
        } else if best_score >= beta {
            ResultKind::LowerBound
        } else {
            ResultKind::Exact
        };

        self.transposition_table[transpo_idx] = Some(SearchResult {
            hash: board_hash,
            depth,
            score: best_score,
            kind,
        });

        best_score
    }
}
