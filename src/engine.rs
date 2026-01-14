use chess::{Board, BoardStatus, ChessMove, MoveGen};
use vampirc_uci::Duration;

use std::collections::HashMap;

// For debugging purpose. Returns the first available legal move.
pub struct AnyMove;

impl<T: EvaluateEngine> SearchEngine<T> for AnyMove {
    fn next_move(&mut self, state: GameState, _time_info: &Option<TimeInfo>) -> Option<ChessMove> {
        MoveGen::new_legal(&state.last_board()).next()
    }

    fn clear_search_state(&mut self) {
        // AnyMove has no state to clear
    }
}

pub trait EvaluateEngine {
    /// Returns a quantized (integer-valued) evaluation of the position, from the side to move perspective
    fn evaluate(state: &GameState) -> i16;
}

pub trait SearchEngine<T: EvaluateEngine> {
    /// Finds the next move to be played given a GameState and  optional time-control information.
    /// Returns an Option because it can technically fail to find a reasonable move.
    /// Default implementation returns the first available legal move
    fn next_move(&mut self, state: GameState, time_info: &Option<TimeInfo>) -> Option<ChessMove>;

    /// Clear search state (killer moves, history, etc.) when setting a new position
    fn clear_search_state(&mut self);

    /// Used to keep searching moves on opponents time.
    /// Default implementation does nothing, and it may be left as is.
    fn ponder(&mut self) {}
}

/// Undo information for a single move
#[derive(Debug, Clone, Copy)]
struct UndoInfo {
    mv: ChessMove,
    prev_board: Board,
}

#[derive(Debug, Clone)]
pub struct GameState {
    /// Current board position
    board: Board,
    /// Stack of undo information (move + previous board state)
    undo_stack: Vec<UndoInfo>,
    /// A map counting the number of times each position was seen so far.
    /// To be used for implementation of the [threefold repetition rule](https://en.wikipedia.org/wiki/Threefold_repetition)
    seen_positions: HashMap<u64, u8>,
}

impl GameState {
    pub fn from_board(board: Board) -> Self {
        let mut s = Self::default();
        s.board = board;
        s
    }

    #[inline(always)]
    pub fn last_board(&self) -> Board {
        self.board
    }

    #[inline]
    pub fn make_move(&mut self, mv: ChessMove) -> u8 {
        // Store undo info
        let undo_info = UndoInfo {
            mv,
            prev_board: self.board,
        };
        self.undo_stack.push(undo_info);

        // Make the move
        self.board = self.board.make_move_new(mv);

        // Update repetition tracking
        let hash = self.board.get_hash();
        let entry = self.seen_positions.entry(hash).or_insert(0);
        *entry += 1;
        *entry
    }

    #[inline]
    pub fn undo_last_move(&mut self) {
        let undo_info = self.undo_stack.pop().unwrap();

        // Decrement repetition count
        let hash = self.board.get_hash();
        if let Some(count) = self.seen_positions.get_mut(&hash) {
            *count -= 1;
        }

        // Restore previous board
        self.board = undo_info.prev_board;
    }

    #[inline]
    pub fn is_draw(&self) -> bool {
        // Check stalemate
        self.board.status() == BoardStatus::Stalemate
    }

    /// Get the current ply count (for mate distance calculation)
    #[inline(always)]
    pub fn ply(&self) -> usize {
        self.undo_stack.len()
    }
}

impl Default for GameState {
    fn default() -> Self {
        let board = Board::default();
        let mut seen_positions = HashMap::with_capacity(128);
        seen_positions.insert(board.get_hash(), 1);

        Self {
            board,
            undo_stack: Vec::with_capacity(128),
            seen_positions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeInfo {
    pub move_time: Option<Duration>,
    pub white_time: Option<Duration>,
    pub black_time: Option<Duration>,
    pub white_increment: Option<Duration>,
    pub black_increment: Option<Duration>,
    pub moves_to_go: Option<u8>,
}
