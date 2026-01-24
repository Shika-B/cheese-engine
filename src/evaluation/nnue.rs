use chess::{
    ALL_SQUARES, BitBoard, Board, BoardStatus, CastleRights, Color, EMPTY, File, Piece, Square,
};
use ort::Error;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::TensorRef;

use crate::engine::{EvaluateEngine, GameState};
use ndarray::Array1;

const MATE_VALUE: i16 = 10;

fn write_castling(arr: &mut Array1<f32>, board: &Board) {
    // White
    match board.castle_rights(Color::White) {
        CastleRights::KingSide => arr[768] = 1.0,
        CastleRights::QueenSide => arr[769] = 1.0,
        CastleRights::Both => {
            arr[768] = 1.0;
            arr[769] = 1.0;
        }
        CastleRights::NoRights => {}
    }

    // Black
    match board.castle_rights(Color::Black) {
        CastleRights::KingSide => arr[770] = 1.0,
        CastleRights::QueenSide => arr[771] = 1.0,
        CastleRights::Both => {
            arr[770] = 1.0;
            arr[771] = 1.0;
        }
        CastleRights::NoRights => {}
    }
}

/// Retourne un `Array1<u8>` de longueur 768 où chaque élément vaut 0 ou 1.
/// L'ordre suit : pour chaque (color, piece_type) on a 64 cases (square index 0..63).
/// - color : 0 = White, 1 = Black
/// - piece_type : 0 = Pawn, 1 = Knight, 2 = Bishop, 3 = Rook, 4 = Queen, 5 = King
pub fn board_to_input(board: &Board) -> Array1<f32> {
    // 768 + 4 castling + 16 en-passant + 1 side to move = 789
    let mut arr = Array1::<f32>::zeros(789);

    // --- position (768) ---
    for &sq in ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(sq) {
            let square_index = sq.to_index() as usize;

            let piece_type = match piece {
                Piece::Pawn => 0usize,
                Piece::Knight => 1usize,
                Piece::Bishop => 2usize,
                Piece::Rook => 3usize,
                Piece::Queen => 4usize,
                Piece::King => 5usize,
            };

            let color = match board.color_on(sq).expect("piece without color") {
                Color::White => 1usize,
                Color::Black => 0usize,
            };

            let idx = square_index + 64 * (piece_type + 6 * color);
            arr[idx] = 1.0;
        }
    }

    // --- castling rights (768..771) ---
    write_castling(&mut arr, board);

    // --- en passant (772..787) ---
    if let Some(ep_sq) = board.en_passant() {
        let sq = ep_sq.to_index() as i32;

        let ep_index = match sq {
            // rank 4 (white just moved)
            24..=31 => sq - 24, // 0..7
            // rank 5 (black just moved)
            32..=39 => sq - 32 + 8, // 8..15
            _ => return arr,        // état invalide / transitoire
        };

        arr[772 + ep_index as usize] = 1.0;
    }


    // --- side to move (788) ---
    arr[788] = match board.side_to_move() {
        Color::White => 1.0,
        Color::Black => 0.0,
    };
    arr
}

pub struct NnueEval {
    model: Session,
}

impl NnueEval {
    pub fn new() -> Result<Self, Error> {
        let model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(
                "C:/Users/monte/Informatique/Chess/rust/cheese-engine/pyNNUE/models/nnue2.onnx",
            )?;
        Ok(Self { model: model })
    }
}

impl EvaluateEngine for NnueEval {
    fn evaluate(&mut self, state: &GameState) -> Result<i16, Error> {
        if state.is_draw() {
            return Ok(0);
        }

        let board = state.last_board();
        let status = board.status();

        if status == BoardStatus::Checkmate {
            return Ok(-MATE_VALUE + state.ply() as i16);
        }

        let input = board_to_input(&board);

        let outputs = self
            .model
            .run(ort::inputs![TensorRef::from_array_view(&input)?])?;
        let predictions = outputs[0].try_extract_array::<f32>()?;
        let score = ((predictions[0] - 0.5) * 100.0) as i16;

        // Return from side to move perspective
        if board.side_to_move() == Color::White {
            Ok(score)
        } else {
            Ok(-score)
        }
    }
}
