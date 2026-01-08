use chess::{Board, Color, Piece};

use crate::engine::EvaluateEngine;





// Simplest position evaluation possible
const PAWN_VALUE: i16 = 100;
const KNIGHT_VALUE: i16 = 320;
const BISHOP_VALUE: i16 = 330;
const ROOK_VALUE: i16 = 500;
const QUEEN_VALUE: i16 = 900;


pub struct CountMaterial;

impl EvaluateEngine for CountMaterial {
    fn evaluate(board: &Board, is_white: bool) -> i16 {
        let mut score = 0;

        // Lambda function to avoid repetive code
        let count = |piece, color| {
            (board
                .pieces(piece)
                 & board.color_combined(color))
                .popcnt() as i16
        };

        score += PAWN_VALUE   * (count(Piece::Pawn,   Color::White) - count(Piece::Pawn,   Color::Black));
        score += KNIGHT_VALUE * (count(Piece::Knight, Color::White) - count(Piece::Knight, Color::Black));
        score += BISHOP_VALUE * (count(Piece::Bishop, Color::White) - count(Piece::Bishop, Color::Black));
        score += ROOK_VALUE   * (count(Piece::Rook,   Color::White) - count(Piece::Rook,   Color::Black));
        score += QUEEN_VALUE  * (count(Piece::Queen,  Color::White) - count(Piece::Queen,  Color::Black));

        let sign = if is_white { 1 } else { -1};
        sign * score
    }
}