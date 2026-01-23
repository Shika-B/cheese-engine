mod nnue;
pub use nnue::NnueEval;
use ort::Error;
pub use pst::PstEval;
mod pst;

use chess::{BoardStatus, Color, Piece};

use crate::engine::{EvaluateEngine, GameState};

// Simplest position evaluation possible
pub const PAWN_VALUE: i16 = 100;
pub const KNIGHT_VALUE: i16 = 320;
pub const BISHOP_VALUE: i16 = 330;
pub const ROOK_VALUE: i16 = 500;
pub const QUEEN_VALUE: i16 = 900;

pub const MATE_VALUE: i16 = 30_000;

pub struct CountMaterial;

impl EvaluateEngine for CountMaterial {
    fn evaluate(&mut self, state: &GameState) -> Result<i16, Error> {
        if state.is_draw() {
            return Ok(0);
        }

        let board = state.last_board();
        let status = board.status();

        if status == BoardStatus::Checkmate {
            return Ok(-MATE_VALUE + state.ply() as i16);
        }

        let mut score = 0;

        let white = board.color_combined(Color::White);
        let black = board.color_combined(Color::Black);

        // Lambda function to avoid repetive code
        let count = |piece| {
            let pieces = board.pieces(piece);
            ((pieces & white).popcnt() - (pieces & black).popcnt()) as i16
        };

        score += PAWN_VALUE * count(Piece::Pawn);
        score += KNIGHT_VALUE * count(Piece::Knight);
        score += BISHOP_VALUE * count(Piece::Bishop);
        score += ROOK_VALUE * count(Piece::Rook);
        score += QUEEN_VALUE * count(Piece::Queen);

        if board.side_to_move() == Color::White {
            Ok(score)
        } else {
            Ok(-score)
        }
    }
}
