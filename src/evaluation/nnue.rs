
use chess::{Board, BoardStatus, Color, Piece, Square, BitBoard, File, EMPTY, ALL_SQUARES};
use ort::Error;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::TensorRef;

use crate::engine::{EvaluateEngine, GameState};
use ndarray::Array1;


const MATE_VALUE: i16 = 10;

/// Retourne un `Vec<u8>` de longueur 768 où chaque élément vaut 0 ou 1.
/// L'ordre suit : pour chaque (color, piece_type) on a 64 cases (square index 0..63).
/// - color : 0 = White, 1 = Black
/// - piece_type : 0 = Pawn, 1 = Knight, 2 = Bishop, 3 = Rook, 4 = Queen, 5 = King
pub fn board_to_768(board: &Board) -> Array1<f32> {
    let mut arr = Array1::<f32>::zeros(768);

    // itérer sur toutes les cases (constante fournie par le crate `chess`)
    for &sq in ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(sq) {
            // numéro de la case (0..63)
            let square_index = sq.to_index() as usize;


            // mappez le type de pièce en 0..5
            let piece_type = match piece {
                Piece::Pawn => 0usize,
                Piece::Knight => 1usize,
                Piece::Bishop => 2usize,
                Piece::Rook => 3usize,
                Piece::Queen => 4usize,
                Piece::King => 5usize,
            };


            // couleur sur la case
            let color = match board.color_on(sq).expect("case avec pièce doit avoir une couleur") {
                Color::White => 1usize,
                Color::Black => 0usize,
            };


            let idx = square_index + 64 * (piece_type + 6 * color);
            arr[idx] = 1.0;
        }
    }
    
    arr
}

pub struct NnueEval {
    model : Session
}

impl NnueEval {
    pub fn new() -> Result<Self, Error> {
        let model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file("C:/Users/monte/Informatique/Chess/rust/cheese-engine/pyNNUE/models/nnue1.onnx")?;
        println!("built nnue");
        Ok(Self {model : model})
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

        let input = board_to_768(&board);

        let outputs = self.model.run(ort::inputs![TensorRef::from_array_view(&input)?])?;
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
