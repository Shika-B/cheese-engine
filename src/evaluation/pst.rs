// TODO: Make the code less repetitive.

use chess::{Board, BoardStatus, Color, Piece, Square, BitBoard, File, EMPTY};
use ort::Error;

use crate::engine::{EvaluateEngine, GameState};

const MATE_VALUE: i16 = 30_000;

// Material values (centipawns)
const PAWN_VALUE: i16 = 100;
const KNIGHT_VALUE: i16 = 320;
const BISHOP_VALUE: i16 = 330;
const ROOK_VALUE: i16 = 500;
const QUEEN_VALUE: i16 = 900;

const BISHOP_PAIR_BONUS: i16 = 50;
const ROOK_OPEN_FILE_BONUS: i16 = 25;
const ROOK_SEMI_OPEN_FILE_BONUS: i16 = 15;
const PASSED_PAWN_BONUS: [i16; 8] = [0, 10, 20, 40, 70, 120, 200, 0]; // By rank
const DOUBLED_PAWN_PENALTY: i16 = -15;
const ISOLATED_PAWN_PENALTY: i16 = -20;
const KING_SAFETY_PAWN_SHIELD: i16 = 10;

// Endgame evaluation tuning constants
const KING_PROXIMITY_BONUS_PER_SQUARE: i16 = 10;  // Bonus for attacking King being close to enemy King
const EDGE_RESTRICTION_BONUS_PER_SQUARE: i16 = 30; // Bonus for enemy King being near edge
const MOBILITY_RESTRICTION_BONUS_PER_SQUARE: i16 = 5; // Bonus per restricted King move
const ENDGAME_ACTIVATION_PHASE: i16 = 200;  // Phase threshold for endgame bonuses
const PURE_ENDGAME_PHASE: i16 = 210;  // Phase threshold for mate progress bonus (lowered to capture Q+K vs K)

// Piece-Square Tables (White's perspective, flipped for Black)
// Values are from White's perspective (rank 0 = 1st rank for White)
// Both midgame and endgame tables are here.

const PAWN_PST_MG: [i16; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

const PAWN_PST_EG: [i16; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     80,  80,  80,  80,  80,  80,  80,  80,
     50,  50,  50,  50,  50,  50,  50,  50,
     30,  30,  30,  30,  30,  30,  30,  30,
     20,  20,  20,  20,  20,  20,  20,  20,
     10,  10,  10,  10,  10,  10,  10,  10,
     10,  10,  10,  10,  10,  10,  10,  10,
      0,   0,   0,   0,   0,   0,   0,   0,
];

const KNIGHT_PST_MG: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const KNIGHT_PST_EG: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_PST_MG: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const BISHOP_PST_EG: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PST_MG: [i16; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

const ROOK_PST_EG: [i16; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

const QUEEN_PST_MG: [i16; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

const QUEEN_PST_EG: [i16; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
     -5,   0,   5,   5,   5,   5,   0,  -5,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

const KING_PST_MG: [i16; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

const KING_PST_EG: [i16; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50,
];

/// Helper struct to avoid repeated mating material calculations
struct EndgameContext {
    white_winning: bool,  // White has mating material and Black has no defense
    black_winning: bool,  // Black has mating material and White has no defense
}

pub struct PstEval;

impl PstEval {
    pub fn new() -> Self {
        Self {}
    }
    /// Calculate game phase (0 = opening, 256 = endgame)
    /// Based on remaining material
    #[inline]
    fn game_phase(board: &Board) -> i16 {
        const KNIGHT_PHASE: i16 = 1;
        const BISHOP_PHASE: i16 = 1;
        const ROOK_PHASE: i16 = 2;
        const QUEEN_PHASE: i16 = 4;
        const TOTAL_PHASE: i16 = KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

        let mut phase = TOTAL_PHASE;

        phase -= (board.pieces(Piece::Knight).popcnt() as i16) * KNIGHT_PHASE;
        phase -= (board.pieces(Piece::Bishop).popcnt() as i16) * BISHOP_PHASE;
        phase -= (board.pieces(Piece::Rook).popcnt() as i16) * ROOK_PHASE;
        phase -= (board.pieces(Piece::Queen).popcnt() as i16) * QUEEN_PHASE;

        // Scale to 0-256
        (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE
    }

    /// Interpolate between middlegame and endgame scores
    #[inline(always)]
    fn interpolate(mg_score: i16, eg_score: i16, phase: i16) -> i16 {
        ((mg_score * (256 - phase)) + (eg_score * phase)) / 256
    }

    /// Get piece-square table value for a piece on a square
    #[inline]
    fn pst_value(square: Square, color: Color, mg_table: &[i16; 64], eg_table: &[i16; 64], phase: i16) -> i16 {
        let idx = if color == Color::White {
            square.to_index()
        } else {
            // Flip square for black
            square.to_index() ^ 56
        };

        Self::interpolate(mg_table[idx], eg_table[idx], phase)
    }

    /// Evaluate material and position using PSTs
    fn evaluate_material_pst(board: &Board, phase: i16) -> i16 {
        let mut score = 0;

        let white = board.color_combined(Color::White);
        let black = board.color_combined(Color::Black);

        let pawns = board.pieces(Piece::Pawn);
        for square in pawns & white {
            score += PAWN_VALUE + Self::pst_value(square, Color::White, &PAWN_PST_MG, &PAWN_PST_EG, phase);
        }
        for square in pawns & black {
            score -= PAWN_VALUE + Self::pst_value(square, Color::Black, &PAWN_PST_MG, &PAWN_PST_EG, phase);
        }

        let knights = board.pieces(Piece::Knight);
        for square in knights & white {
            score += KNIGHT_VALUE + Self::pst_value(square, Color::White, &KNIGHT_PST_MG, &KNIGHT_PST_EG, phase);
        }
        for square in knights & black {
            score -= KNIGHT_VALUE + Self::pst_value(square, Color::Black, &KNIGHT_PST_MG, &KNIGHT_PST_EG, phase);
        }

        let bishops = board.pieces(Piece::Bishop);
        for square in bishops & white {
            score += BISHOP_VALUE + Self::pst_value(square, Color::White, &BISHOP_PST_MG, &BISHOP_PST_EG, phase);
        }
        for square in bishops & black {
            score -= BISHOP_VALUE + Self::pst_value(square, Color::Black, &BISHOP_PST_MG, &BISHOP_PST_EG, phase);
        }

        let rooks = board.pieces(Piece::Rook);
        for square in rooks & white {
            score += ROOK_VALUE + Self::pst_value(square, Color::White, &ROOK_PST_MG, &ROOK_PST_EG, phase);
        }
        for square in rooks & black {
            score -= ROOK_VALUE + Self::pst_value(square, Color::Black, &ROOK_PST_MG, &ROOK_PST_EG, phase);
        }

        let queens = board.pieces(Piece::Queen);
        for square in queens & white {
            score += QUEEN_VALUE + Self::pst_value(square, Color::White, &QUEEN_PST_MG, &QUEEN_PST_EG, phase);
        }
        for square in queens & black {
            score -= QUEEN_VALUE + Self::pst_value(square, Color::Black, &QUEEN_PST_MG, &QUEEN_PST_EG, phase);
        }

        // Kings (no material value, just positional)
        let king_sq = (board.pieces(Piece::King) & white).to_square();
        score += Self::pst_value(king_sq, Color::White, &KING_PST_MG, &KING_PST_EG, phase);

        let king_sq = (board.pieces(Piece::King) & black).to_square();
        score -= Self::pst_value(king_sq, Color::Black, &KING_PST_MG, &KING_PST_EG, phase);

        score
    }

    /// passed pawn: no enemy pawns in front or on adjacent files)
    #[inline]
    fn is_passed_pawn(square: Square, color: Color, board: &Board) -> bool {
        let file = square.get_file().to_index();
        let rank = square.get_rank().to_index();

        let enemy_pawns = board.pieces(Piece::Pawn) & board.color_combined(!color);

        // mask for blocking pawns
        let blocking_files = if file == 0 {
            0b01100000_01100000_01100000_01100000_01100000_01100000_01100000_01100000u64
        } else if file == 7 {
            0b00000110_00000110_00000110_00000110_00000110_00000110_00000110_00000110u64
        } else {
            let left_file = 1u64 << (file - 1);
            let center_file = 1u64 << file;
            let right_file = 1u64 << (file + 1);
            let file_mask = left_file | center_file | right_file;
            file_mask * 0x0101010101010101u64
        };

        // Define rank mask (squares in front)
        let rank_mask = if color == Color::White {
            // All ranks above
            !((1u64 << ((rank + 1) * 8)) - 1)
        } else {
            // All ranks below
            (1u64 << (rank * 8)) - 1
        };

        let passed_mask = BitBoard(blocking_files & rank_mask);
        (enemy_pawns & passed_mask) == EMPTY
    }

    /// Evaluate pawn structure
    fn evaluate_pawns(board: &Board) -> i16 {
        let mut score = 0;

        let white_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
        let black_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);

        for square in white_pawns {
            let file = square.get_file().to_index();
            let rank = square.get_rank().to_index();

            // Passed pawn bonus
            if Self::is_passed_pawn(square, Color::White, board) {
                score += PASSED_PAWN_BONUS[rank];
            }

            // Doubled pawns
            let file_mask = chess::get_file(square.get_file());
            if (white_pawns & file_mask).popcnt() > 1 {
                score += DOUBLED_PAWN_PENALTY;
            }

            // Isolated pawns (no friendly pawns on adjacent files)
            let adjacent_files = if file == 0 {
                chess::get_file(File::B)
            } else if file == 7 {
                chess::get_file(File::G)
            } else {
                chess::get_file(File::from_index((file - 1) as usize)) | chess::get_file(File::from_index((file + 1) as usize))
            };

            if (white_pawns & adjacent_files) == EMPTY {
                score += ISOLATED_PAWN_PENALTY;
            }
        }

        for square in black_pawns {
            let file = square.get_file().to_index();
            let rank = square.get_rank().to_index();

            // Passed pawn bonus (flipped rank for black)
            if Self::is_passed_pawn(square, Color::Black, board) {
                score -= PASSED_PAWN_BONUS[7 - rank];
            }

            // Doubled pawns
            let file_mask = chess::get_file(square.get_file());
            if (black_pawns & file_mask).popcnt() > 1 {
                score -= DOUBLED_PAWN_PENALTY;
            }

            // Isolated pawns
            let adjacent_files = if file == 0 {
                chess::get_file(File::B)
            } else if file == 7 {
                chess::get_file(File::G)
            } else {
                chess::get_file(File::from_index((file - 1) as usize)) | chess::get_file(File::from_index((file + 1) as usize))
            };

            if (black_pawns & adjacent_files) == EMPTY {
                score -= ISOLATED_PAWN_PENALTY;
            }
        }

        score
    }

    fn evaluate_bishops(board: &Board) -> i16 {
        let mut score = 0;

        let white_bishops = board.pieces(Piece::Bishop) & board.color_combined(Color::White);
        let black_bishops = board.pieces(Piece::Bishop) & board.color_combined(Color::Black);

        // Bishop pair bonus
        if white_bishops.popcnt() >= 2 {
            score += BISHOP_PAIR_BONUS;
        }
        if black_bishops.popcnt() >= 2 {
            score -= BISHOP_PAIR_BONUS;
        }

        score
    }

    fn evaluate_rooks(board: &Board) -> i16 {
        let mut score = 0;

        let white_rooks = board.pieces(Piece::Rook) & board.color_combined(Color::White);
        let black_rooks = board.pieces(Piece::Rook) & board.color_combined(Color::Black);
        let white_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
        let black_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);

        // White rooks on open/semi-open files
        for square in white_rooks {
            let file_mask = chess::get_file(square.get_file());
            let has_white_pawns = (white_pawns & file_mask) != EMPTY;
            let has_black_pawns = (black_pawns & file_mask) != EMPTY;

            if !has_white_pawns && !has_black_pawns {
                score += ROOK_OPEN_FILE_BONUS;
            } else if !has_white_pawns {
                score += ROOK_SEMI_OPEN_FILE_BONUS;
            }
        }

        // Black rooks on open/semi-open files
        for square in black_rooks {
            let file_mask = chess::get_file(square.get_file());
            let has_white_pawns = (white_pawns & file_mask) != EMPTY;
            let has_black_pawns = (black_pawns & file_mask) != EMPTY;

            if !has_white_pawns && !has_black_pawns {
                score -= ROOK_OPEN_FILE_BONUS;
            } else if !has_black_pawns {
                score -= ROOK_SEMI_OPEN_FILE_BONUS;
            }
        }

        score
    }

    /// Evaluate king safety in middlegame
    fn evaluate_king_safety(board: &Board, phase: i16) -> i16 {
        // Only relevant in middlegame
        if phase > 180 {
            return 0;
        }

        let mut score = 0;

        let white_king = (board.pieces(Piece::King) & board.color_combined(Color::White)).to_square();
        let black_king = (board.pieces(Piece::King) & board.color_combined(Color::Black)).to_square();
        let white_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
        let black_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);

        // White king pawn shield
        let white_shield_squares = chess::get_king_moves(white_king);
        for sq in white_shield_squares {
            if (white_pawns & BitBoard::from_square(sq)) != EMPTY {
                score += KING_SAFETY_PAWN_SHIELD;
            }
        }

        // Black king pawn shield
        let black_shield_squares = chess::get_king_moves(black_king);
        for sq in black_shield_squares {
            if (black_pawns & BitBoard::from_square(sq)) != EMPTY {
                score -= KING_SAFETY_PAWN_SHIELD;
            }
        }

        // Scale by game phase (less important in endgame)
        score * (256 - phase) / 256
    }

    /// Check if a color has mating material
    #[inline]
    fn has_mating_material(board: &Board, color: Color) -> bool {
        let pieces = board.color_combined(color);
        let queens = (board.pieces(Piece::Queen) & pieces).popcnt();
        let rooks = (board.pieces(Piece::Rook) & pieces).popcnt();
        let minors = ((board.pieces(Piece::Knight) | board.pieces(Piece::Bishop)) & pieces).popcnt();
        let pawns = (board.pieces(Piece::Pawn) & pieces).popcnt();

        queens > 0 || rooks > 0 || minors >= 2 || pawns > 0
    }

    /// Check if a color has defensive material (anything beyond a bare King)
    #[inline]
    fn has_defensive_material(board: &Board, color: Color) -> bool {
        let pieces = board.color_combined(color);
        let queens = (board.pieces(Piece::Queen) & pieces).popcnt();
        let rooks = (board.pieces(Piece::Rook) & pieces).popcnt();
        let minors = ((board.pieces(Piece::Knight) | board.pieces(Piece::Bishop)) & pieces).popcnt();
        let pawns = (board.pieces(Piece::Pawn) & pieces).popcnt();

        queens > 0 || rooks > 0 || minors > 0 || pawns > 0
    }

    /// Calculate Manhattan distance between two squares
    #[inline]
    fn manhattan_distance(sq1: Square, sq2: Square) -> i16 {
        let file1 = sq1.get_file().to_index() as i16;
        let rank1 = sq1.get_rank().to_index() as i16;
        let file2 = sq2.get_file().to_index() as i16;
        let rank2 = sq2.get_rank().to_index() as i16;

        (file1 - file2).abs() + (rank1 - rank2).abs()
    }

    /// Calculate distance of a square to the nearest edge
    #[inline]
    fn edge_distance(sq: Square) -> i16 {
        let file = sq.get_file().to_index() as i16;
        let rank = sq.get_rank().to_index() as i16;

        let file_dist = file.min(7 - file);
        let rank_dist = rank.min(7 - rank);

        file_dist.min(rank_dist)
    }

    /// Analyze if either side is in a winning mating endgame
    #[inline]
    fn analyze_endgame(board: &Board) -> EndgameContext {
        EndgameContext {
            white_winning: Self::has_mating_material(board, Color::White)
                           && !Self::has_defensive_material(board, Color::Black),
            black_winning: Self::has_mating_material(board, Color::Black)
                           && !Self::has_defensive_material(board, Color::White),
        }
    }

    /// Evaluate King proximity in endgames with mating material
    fn evaluate_king_proximity(board: &Board, phase: i16, context: &EndgameContext) -> i16 {
        // Only relevant in late endgame
        if phase < ENDGAME_ACTIVATION_PHASE {
            return 0;
        }

        let mut score = 0;

        if context.white_winning {
            // White is trying to mate Black
            let white_king = (board.pieces(Piece::King) & board.color_combined(Color::White)).to_square();
            let black_king = (board.pieces(Piece::King) & board.color_combined(Color::Black)).to_square();

            let distance = Self::manhattan_distance(white_king, black_king);

            // Bonus for Kings being close (max 70cp at distance 0)
            score += (7 - distance.min(7)) * KING_PROXIMITY_BONUS_PER_SQUARE;
        }

        if context.black_winning {
            // Black is trying to mate White
            let white_king = (board.pieces(Piece::King) & board.color_combined(Color::White)).to_square();
            let black_king = (board.pieces(Piece::King) & board.color_combined(Color::Black)).to_square();

            let distance = Self::manhattan_distance(white_king, black_king);

            // Same logic for black
            score -= (7 - distance.min(7)) * KING_PROXIMITY_BONUS_PER_SQUARE;
        }

        score
    }

    /// Evaluate enemy King restriction to edges in mating endgames
    fn evaluate_king_edge_restriction(board: &Board, phase: i16, context: &EndgameContext) -> i16 {
        // Only relevant in late endgame
        if phase < ENDGAME_ACTIVATION_PHASE {
            return 0;
        }

        let mut score = 0;

        if context.white_winning {
            let black_king = (board.pieces(Piece::King) & board.color_combined(Color::Black)).to_square();
            let edge_dist = Self::edge_distance(black_king);

            // Big bonus for enemy King near edges (90cp when on edge)
            score += (3 - edge_dist.min(3)) * EDGE_RESTRICTION_BONUS_PER_SQUARE;
        }

        if context.black_winning {
            let white_king = (board.pieces(Piece::King) & board.color_combined(Color::White)).to_square();
            let edge_dist = Self::edge_distance(white_king);

            score -= (3 - edge_dist.min(3)) * EDGE_RESTRICTION_BONUS_PER_SQUARE;
        }

        score
    }

    /// Estimate mate distance and give bonus for positions closer to mate
    fn evaluate_mate_progress(board: &Board, phase: i16, context: &EndgameContext) -> i16 {
        // Only in pure endgames
        if phase < PURE_ENDGAME_PHASE {
            return 0;
        }

        let mut score = 0;

        // If winning side, add bonus based on restricting King mobility
        if context.white_winning {
            // Count mobility of black King (fewer moves = closer to mate)
            let black_king = (board.pieces(Piece::King) & board.color_combined(Color::Black)).to_square();
            let king_moves = chess::get_king_moves(black_king);

            // Filter out squares occupied by black pieces or attacked by white
            let black = board.color_combined(Color::Black);
            let white = board.color_combined(Color::White);

            // Simple mobility check: exclude squares with black pieces
            // (A full attack detection would be more accurate but slower)
            let legal_king_squares = king_moves & !black & !white;
            let legal_king_moves = legal_king_squares.popcnt() as i16;

            // Bonus for restricting King mobility (5cp per restricted square)
            score += (8 - legal_king_moves) * MOBILITY_RESTRICTION_BONUS_PER_SQUARE;
        }

        if context.black_winning {
            let white_king = (board.pieces(Piece::King) & board.color_combined(Color::White)).to_square();
            let king_moves = chess::get_king_moves(white_king);

            let black = board.color_combined(Color::Black);
            let white = board.color_combined(Color::White);

            let legal_king_squares = king_moves & !black & !white;
            let legal_king_moves = legal_king_squares.popcnt() as i16;

            score -= (8 - legal_king_moves) * MOBILITY_RESTRICTION_BONUS_PER_SQUARE;
        }

        score
    }
}

impl EvaluateEngine for PstEval {
    fn evaluate(&mut self, state: &GameState) -> Result<i16, Error> {
        if state.is_draw() {
            return Ok(0);
        }

        let board = state.last_board();
        let status = board.status();

        if status == BoardStatus::Checkmate {
            return Ok(-MATE_VALUE + state.ply() as i16);
        }

        let phase = Self::game_phase(&board);

        let mut score = 0;

        // Material + PST evaluation
        score += Self::evaluate_material_pst(&board, phase);

        // Pawn structure
        score += Self::evaluate_pawns(&board);

        // Bishop evaluation
        score += Self::evaluate_bishops(&board);

        // Rook evaluation
        score += Self::evaluate_rooks(&board);

        // King safety
        score += Self::evaluate_king_safety(&board, phase);

        // Endgame-specific evaluation (compute context once, use for all three functions)
        let endgame_context = Self::analyze_endgame(&board);
        score += Self::evaluate_king_proximity(&board, phase, &endgame_context);
        score += Self::evaluate_king_edge_restriction(&board, phase, &endgame_context);
        score += Self::evaluate_mate_progress(&board, phase, &endgame_context);

        // Return from side to move perspective
        if board.side_to_move() == Color::White {
            Ok(score)
        } else {
            Ok(-score)
        }
    }
}
