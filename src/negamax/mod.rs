mod mv_iter;

use std::{i16, time::Instant};

use chess::{BoardStatus, ChessMove, EMPTY, MoveGen};

use crate::{
    engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo},
    negamax::mv_iter::MvIter,
};

const TRANSPOTION_TABLE_SIZE: usize = 16_777_216; // 16_777_216 = 2^24

const MATE_THRESHOLD: i16 = 29_000;

const MAX_DEPTH: u16 = 4;

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
    best_move: Option<ChessMove>,
}

pub struct Negamax {
    nodes_explored: usize,
    transposition_table: Vec<SearchResult>,
}

impl<E: EvaluateEngine> SearchEngine<E> for Negamax {
    fn next_move(
        &mut self,
        mut state: GameState,
        _time_info: &Option<TimeInfo>,
    ) -> Option<ChessMove> {
        self.nodes_explored = 0;
        let start = Instant::now();

        let board = state.last_board();

        let mut best_move = None;
        let mut last_score = 0;
        for curr_depth in 1..=MAX_DEPTH {
            log::debug!("DEPTH {}", curr_depth);
            let mut window = 32;
            let mut best_score = -i16::MAX;

            loop {
                let alpha_orig = last_score - window;
                // let alpha_orig = -i16::MAX;

                let mut alpha = alpha_orig;
                let beta = last_score + window;
                // let beta = i16::MAX;

                let explore_first = [best_move];
                let mv_iter = MvIter::new(&explore_first, &board);
                
                let mut aspiration_failed = false;
                for mv in mv_iter {
                    state.make_move(mv);
                    let score = -self.search_eval::<E>(&mut state, -beta, -alpha, curr_depth);
                    state.undo_last_move();

                    if curr_depth == MAX_DEPTH {
                        log::debug!(
                            "Considering mv {} at depth {} with score {}",
                            mv,
                            curr_depth,
                            score
                        );
                    }
                    if score > best_score {
                        best_score = score;
                        best_move = Some(mv)
                    }
                    if best_score <= alpha_orig || best_score >= beta {
                        window *= 2;
                        aspiration_failed = true;
                        break 
                    }
                    alpha = alpha.max(best_score);
                }

                if !aspiration_failed {
                    last_score = best_score;
                    break;
                }
            }
        }

        let elapsed = Instant::now() - start;
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
            transposition_table: vec![SearchResult::default(); TRANSPOTION_TABLE_SIZE],
        }
    }

    pub fn search_eval<E: EvaluateEngine>(
        &mut self,
        state: &mut GameState,
        mut alpha: i16,
        beta: i16,
        depth: u16,
    ) -> i16 {
        self.nodes_explored += 1;
        let board = state.last_board();
        let board_hash = board.get_hash();
        let transpo_idx = (board_hash as usize) & (TRANSPOTION_TABLE_SIZE - 1);

        let entry = self.transposition_table[transpo_idx];

        let mut best_score = -i16::MAX;
        let mut best_move = None;

        /*  if entry.hash == board_hash && entry.depth >= depth {
                    match entry.kind {
                        ResultKind::Exact => return entry.score,
                        ResultKind::LowerBound if entry.score >= beta => return entry.score,
                        ResultKind::UpperBound if entry.score <= alpha => return entry.score,
                        _ => {}
                    }
                    if let Some(mv) = entry.best_move {
                        best_move = Some(mv);
                    }
                }
        */

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
        let explore_first = [best_move];

        let mv_iter = MvIter::new(&explore_first, &board);
        for mv in mv_iter {
            state.make_move(mv);
            let score = -self.search_eval::<E>(state, -beta, -alpha, depth - 1);
            state.undo_last_move();

            if score > best_score {
                best_score = score;
                best_move = Some(mv)
            }
            alpha = alpha.max(best_score);

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

        self.transposition_table[transpo_idx] = SearchResult {
            hash: board_hash,
            depth,
            score: best_score,
            kind,
            best_move,
        };

        best_score
    }
}
