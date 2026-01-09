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
    fn evaluate(state: &GameState) -> i16 {
        if state.is_draw() {
            return 0;
        }

        let board = state.last_board();
        let status = board.status();

        if status == BoardStatus::Checkmate {
            return -MATE_VALUE + state.num_moves as i16;
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
            score
        } else {
            -score
        }
    }
}
