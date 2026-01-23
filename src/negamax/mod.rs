mod mv_iter;

use std::{
    i16,
    time::{Duration, Instant},
};

use chess::{BoardStatus, ChessMove, MoveGen};
use chrono::{TimeDelta, Utc};
use ort::Error;

use crate::{
    engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo},
    negamax::mv_iter::StagedMoveIterator,
};

const TRANSPOTION_TABLE_SIZE: usize = 16_777_216; // 16_777_216 = 2^24

const MATE_THRESHOLD: i16 = 29_000;

const MAX_DEPTH: u16 = 4;
const MAX_PLY: usize = 128;
const REPETITION_PENALTY: i16 = 50;

// Endgame quiescence constants
const SIMPLE_ENDGAME_PIECE_COUNT: u32 = 4;
const ENDGAME_QSEARCH_DEPTH: usize = 8;

#[derive(Debug, PartialEq, Copy, Clone)]
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

pub struct Negamax<E : EvaluateEngine> {
    nodes_explored: usize,
    transposition_table: Vec<SearchResult>,

    // Move ordering heuristics
    killer_moves: [[Option<ChessMove>; 2]; MAX_PLY],
    counter_moves: [Option<ChessMove>; 64],
    history_table: [[i32; 64]; 64],
    history_move_count: u32,

    evaluator : E
}



impl<E: EvaluateEngine> SearchEngine<E> for Negamax<E> {
    fn next_move(&mut self, mut state: GameState, _time_info: TimeInfo) -> Option<ChessMove> {
        self.nodes_explored = 0;
        let start = Utc::now();
        let mut elapsed = TimeDelta::zero();

        // Clear killer moves for new search
        for ply in 0..MAX_PLY {
            self.killer_moves[ply] = [None; 2];
        }

        let board = state.last_board();
        let board_hash = board.get_hash();
        let mut best_move = None;
        let mut last_score = 0;

        let mut start_depth = 1;

        let entry = self.get_tt_entry(board_hash);
        if let Some(entry) = entry {
            match entry.kind {
                ResultKind::Exact => start_depth = entry.depth + 1,
                ResultKind::LowerBound => (),
                ResultKind::UpperBound => (),
                _ => {}
            }
            if let Some(mv) = entry.best_move {
                best_move = Some(mv);
            }
        }
        let mut last_depth = 0;
        for curr_depth in start_depth..=MAX_DEPTH {
            last_depth = curr_depth;
            let mut window = 32;
            let mut alpha_orig = last_score - window;
            let mut beta = last_score + window;

            loop {
                let mv_iter = StagedMoveIterator::new(
                    board,
                    best_move,
                    &self.killer_moves[0],
                    None, // No counter move at root
                    &self.history_table,
                );

                let mut best_score = -i16::MAX;

                let mut alpha = alpha_orig;

                for mv in mv_iter {
                    let repetition_count = state.make_move(mv);
                    let score = if repetition_count >= 3 {
                        0
                    } else {
                        -self.search_eval(&mut state, -beta, -alpha, curr_depth - 1, 1)
                    };

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
                if best_score <= alpha_orig {
                    alpha_orig = alpha_orig.saturating_sub(window);
                } else if best_score >= beta {
                    beta = beta.saturating_add(window);
                } else {
                    last_score = best_score;
                    break;
                }
                window *= 2;
            }
            elapsed = (Utc::now() - start);
        }

        log::info!(
            "Nodes explored: {} in {}ms. {:.0} NPS up to depth {}",
            self.nodes_explored,
            elapsed.num_milliseconds(),
            (self.nodes_explored as f64 / elapsed.as_seconds_f64()).round(),
            last_depth
        );
        log::info!("Best move: {:?}, Best score: {}", best_move.map(|x| x.to_string()), last_score);

        best_move
    }

    fn clear_search_state(&mut self) {
        self.killer_moves = [[None; 2]; MAX_PLY];
        self.counter_moves = [None; 64];
        self.history_table = [[0; 64]; 64];
        self.history_move_count = 0;
        // Clear TT to avoid using moves from previous positions
        self.transposition_table = vec![SearchResult::default(); TRANSPOTION_TABLE_SIZE];
    }
}

impl<E : EvaluateEngine> Negamax<E> {
    pub fn new(evaluator : E) -> Self {
        Self {
            nodes_explored: 0,
            transposition_table: vec![SearchResult::default(); TRANSPOTION_TABLE_SIZE],
            killer_moves: [[None; 2]; MAX_PLY],
            counter_moves: [None; 64],
            history_table: [[0; 64]; 64],
            history_move_count: 0,
            evaluator : evaluator
        }
    }

    fn get_tt_entry(&self, hash: u64) -> Option<SearchResult> {
        let transpo_idx = (hash as usize) & (TRANSPOTION_TABLE_SIZE - 1);

        let entry = self.transposition_table[transpo_idx];
        if entry.hash == hash && entry.kind != ResultKind::None {
            return Some(entry);
        }
        None
    }

    fn save_tt_entry(&mut self, search_result: SearchResult) {
        let transpo_idx = (search_result.hash as usize) & (TRANSPOTION_TABLE_SIZE - 1);
        self.transposition_table[transpo_idx] = search_result;
    }
    pub fn search_eval(
        &mut self,
        state: &mut GameState,
        mut alpha: i16,
        beta: i16,
        depth: u16,
        ply: usize,
    ) -> i16 {
        self.nodes_explored += 1;

        let board = state.last_board();
        let board_hash = board.get_hash();

        let entry = self.get_tt_entry(board_hash);

        let mut best_score = -i16::MAX;
        let mut best_move = None;

        let mut replace_entry = entry.is_none();
        if let Some(entry) = entry {
            replace_entry |= entry.depth <= depth;

            if entry.depth >= depth {
                match entry.kind {
                    ResultKind::Exact => return entry.score,
                    ResultKind::LowerBound if entry.score >= beta => return entry.score,
                    ResultKind::UpperBound if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
            if let Some(mv) = entry.best_move {
                best_move = Some(mv);
            }
        }

        if depth == 0 {
            return self.quiescence(state, alpha, beta, ply);
        }

        match board.status() {
            BoardStatus::Stalemate | BoardStatus::Checkmate => {
                return self.evaluator.evaluate(state).unwrap();
            }
            BoardStatus::Ongoing => (),
        }

        // Store so that we can decide later the kind of bound we want to store in the transposition table
        let alpha_orig = alpha.clone();

        // Determine counter move (response to opponent's last move)
        let counter_move = None; // Will be implemented in Phase 4

        let mv_iter = StagedMoveIterator::new(
            board,
            best_move,
            &self.killer_moves[ply],
            counter_move,
            &self.history_table,
        );

        let mut move_count = 0;

        for mv in mv_iter {
            let repetition_count = state.make_move(mv);

            move_count += 1;

            let score = if repetition_count >= 3 {
                0
            } else if move_count == 1 {
                // First move: full window (PV node)
                -self.search_eval(state, -beta, -alpha, depth - 1, ply + 1)
            } else {
                // Null window search
                let mut score =
                    -self.search_eval(state, -alpha - 1, -alpha, depth - 1, ply + 1);

                // Re-search if it beat alpha
                if score > alpha && score < beta {
                    score = -self.search_eval(state, -beta, -alpha, depth - 1, ply + 1);
                }
                score
            };

            state.undo_last_move();

            if score > best_score {
                best_score = score;
                best_move = Some(mv)
            }
            alpha = alpha.max(best_score);

            if alpha >= beta {
                // Beta cutoff: update move ordering heuristics
                self.update_move_ordering(mv, &board, depth, ply, None);
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

        if replace_entry {
            self.save_tt_entry(SearchResult {
                hash: board_hash,
                depth,
                score: best_score,
                kind,
                best_move,
            });
        }
        best_score
    }

    fn quiescence(
        &mut self,
        state: &mut GameState,
        mut alpha: i16,
        beta: i16,
        ply: usize,
    ) -> i16 {
        self.nodes_explored += 1;

        let stand_pat = self.evaluator.evaluate(state).unwrap();

        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let board = state.last_board();

        // Detect simple mating endgames
        let total_pieces = board.combined().popcnt();
        let is_simple_endgame = total_pieces <= SIMPLE_ENDGAME_PIECE_COUNT;

        // Generate captures
        let mut captures: Vec<ChessMove> = MoveGen::new_legal(&board)
            .filter(|mv| board.piece_on(mv.get_dest()).is_some())
            .collect();

        // Filter out losing captures using SEE
        captures.retain(|&mv| self.see(&board, mv, -100));

        // Sort by MVV-LVV descending
        captures.sort_unstable_by_key(|&mv| -self.mvv_lvv_score(mv, &board));

        // In simple endgames, also consider checks and King moves
        let mut extended_moves = Vec::new();
        if is_simple_endgame && ply < ENDGAME_QSEARCH_DEPTH {
            for mv in MoveGen::new_legal(&board) {
                // Skip captures (already in captures list)
                if board.piece_on(mv.get_dest()).is_some() {
                    continue;
                }

                // Include checks
                let new_board = board.make_move_new(mv);
                if new_board.checkers().popcnt() > 0 {
                    extended_moves.push(mv);
                    continue;
                }

                // Include King moves
                let piece = board.piece_on(mv.get_source());
                if piece == Some(chess::Piece::King) {
                    extended_moves.push(mv);
                }
            }
        }

        // First try captures
        for mv in captures {
            // Delta pruning: if even best-case capture can't raise alpha, skip
            let optimistic_score = stand_pat + self.mvv_lvv_score(mv, &board) + 200;
            if optimistic_score < alpha {
                continue;
            }

            let repetition_count = state.make_move(mv);
            let score = if repetition_count >= 3 {
                0
            } else {
                -self.quiescence(state, -beta, -alpha, ply + 1)
            };
            state.undo_last_move();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        // Then try extended moves in endgame
        for mv in extended_moves {
            let repetition_count = state.make_move(mv);
            let score = if repetition_count >= 3 {
                0
            } else {
                -self.quiescence(state, -beta, -alpha, ply + 1)
            };
            state.undo_last_move();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    #[inline(always)]
    fn mvv_lvv_score(&self, mv: ChessMove, board: &chess::Board) -> i16 {
        // Piece values for MVV-LVV
        const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0]; // P, N, B, R, Q, K

        let victim = board.piece_on(mv.get_dest());
        let attacker = board.piece_on(mv.get_source());

        let mut score = if let Some(victim_piece) = victim {
            let victim_value = PIECE_VALUES[victim_piece.to_index()];
            let attacker_value = if let Some(attacker_piece) = attacker {
                PIECE_VALUES[attacker_piece.to_index()]
            } else {
                0
            };
            // Victim value * 16 ensures victims sorted first, attacker breaks ties
            victim_value * 16 - attacker_value
        } else {
            0
        };

        // Add promotion bonus (promotion piece value - pawn value)
        if let Some(promotion) = mv.get_promotion() {
            let promotion_value = PIECE_VALUES[promotion.to_index()];
            score += promotion_value - PIECE_VALUES[0]; // promotion_value - PAWN_VALUE
        }

        score
    }

    #[inline(always)]
    fn is_quiet_move(&self, mv: ChessMove, board: &chess::Board) -> bool {
        board.piece_on(mv.get_dest()).is_none()
    }

    #[inline(never)]
    fn age_history(&mut self) {
        for i in 0..64 {
            for j in 0..64 {
                self.history_table[i][j] /= 2;
            }
        }
    }

    #[inline(always)]
    fn update_move_ordering(
        &mut self,
        mv: ChessMove,
        board: &chess::Board,
        depth: u16,
        ply: usize,
        last_move_to_square: Option<usize>,
    ) {
        // Only update for quiet moves (captures ordered by MVV-LVV/SEE)
        if self.is_quiet_move(mv, board) {
            // 1. Update killer moves (shift down if new)
            if self.killer_moves[ply][0] != Some(mv) {
                self.killer_moves[ply][1] = self.killer_moves[ply][0];
                self.killer_moves[ply][0] = Some(mv);
            }

            // 2. Update history heuristic (bonus = depth^2)
            let bonus = (depth as i32) * (depth as i32);
            let from = mv.get_source().to_index();
            let to = mv.get_dest().to_index();
            self.history_table[from][to] += bonus;

            // 3. Age history periodically
            self.history_move_count += 1;
            if self.history_move_count >= 1024 {
                self.age_history();
                self.history_move_count = 0;
            }

            // 4. Update counter move
            if let Some(last_to) = last_move_to_square {
                self.counter_moves[last_to] = Some(mv);
            }
        }
    }

    #[inline]
    fn see(&self, board: &chess::Board, mv: ChessMove, threshold: i16) -> bool {
        // Simplified Static Exchange Evaluation
        // Returns true if capture wins at least 'threshold' material

        // Get piece values
        const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0]; // P, N, B, R, Q, K

        let victim = board.piece_on(mv.get_dest());
        let attacker = board.piece_on(mv.get_source());

        if victim.is_none() {
            // Not a capture
            return threshold <= 0;
        }

        if attacker.is_none() {
            return false;
        }

        let victim_value = PIECE_VALUES[victim.unwrap().to_index()];
        let attacker_value = PIECE_VALUES[attacker.unwrap().to_index()];

        // Simple heuristic: capture is good if we gain material
        // victim_value - attacker_value >= threshold
        // This is a simplification - proper SEE would simulate full exchange
        let simple_gain = victim_value - attacker_value;

        // For now, just use this simple heuristic
        // A proper SEE implementation would require more complex attack detection
        simple_gain >= threshold
    }
}
