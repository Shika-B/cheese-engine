use chess::{Board, ChessMove, Color, MoveGen};
use vampirc_uci::Duration;

use std::{collections::HashMap};


// For debugging purpose. Returns the first available legal move.
pub struct AnyMove;


impl<T: EvaluateEngine> SearchEngine<T> for AnyMove {
    fn next_move(&mut self, board: &Board, _state: &GameState, _time_info: Option<TimeInfo>) -> Option<ChessMove> {
        MoveGen::new_legal(board).next()
    }
}


pub trait EvaluateEngine {
    fn evaluate(&self, board: &Board, is_white: bool) -> i16;
}

pub trait SearchEngine<T: EvaluateEngine> {
    /// Finds the next move to be played given a GameState and  optional time-control information.
    /// Returns an Option because it can technically fail to find a reasonable move.
    /// Default implementation returns the first available legal move
    fn next_move(&mut self, board: &Board, state: &GameState, time_info: Option<TimeInfo>) -> Option<ChessMove>;

    /// Used to keep searching moves on opponents time.
    /// Default implementation does nothing, and it may be left as is.
    fn ponder(&mut self) {}
}


pub struct GameState {
    /// Total number of moves played in the game
    pub num_moves: u16,
    /// Total number of moves played since the last piece got captured
    /// To be used for implementation of the [50 moves rule](https://en.wikipedia.org/wiki/Fifty-move_rule)
    pub moves_since_capture: u16,
    /// A map counting the number of times each position was seen so far.
    /// To be used for implementation of the [threefold repetition rule](https://en.wikipedia.org/wiki/Threefold_repetition)  
    pub seen_positions: HashMap<u64, usize>,
    /// Whether or not threefold draw is possible
    pub threefold_available: bool,
}

impl GameState {
    pub fn update_from_move(&mut self, board: Board, mv: ChessMove) -> Board {
        if let Some(_) = board.color_on(mv.get_dest()) {
            self.moves_since_capture = 0;
        }
        self.num_moves += 1;
        let board = board.make_move_new(mv);

        let count = self.seen_positions.entry(board.get_hash()).or_insert(0);
        *count += 1;
        
        if *count >= 3 {
            self.threefold_available = true
        }
        board  
    }
    pub fn can_draw(&self) -> bool {
        return self.moves_since_capture >= 50 || self.threefold_available 
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            num_moves: 0,
            moves_since_capture: 0,
            // Costs basically nothing to preallocate
            seen_positions: HashMap::with_capacity(128), 
            threefold_available: false,
        }
    }
}  
pub struct TimeInfo {
    move_time: Option<Duration>,
    white_time: Option<Duration>,
    black_time: Option<Duration>,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
    moves_to_go: Option<u8>,
}
