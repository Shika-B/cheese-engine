use crate::engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo};
use chess::{Board, BoardStatus, ChessMove};
use std::str::FromStr;

/// Represents the outcome of a chess game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// White wins
    WhiteWins,
    /// Black wins
    BlackWins,
    /// Draw (stalemate, threefold repetition, or fifty-move rule)
    Draw,
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::WhiteWins => write!(f, "1-0"),
            GameResult::BlackWins => write!(f, "0-1"),
            GameResult::Draw => write!(f, "1/2-1/2"),
        }
    }
}

/// Represents a chess game in PGN (Portable Game Notation) format
#[derive(Debug, Clone)]
pub struct Pgn {
    /// Starting FEN position
    pub fen: String,
    /// List of moves in SAN (Standard Algebraic Notation)
    pub moves: Vec<String>,
    /// Game result
    pub result: GameResult,
    /// Optional game metadata (event, site, date, etc.)
    pub tags: Vec<(String, String)>,
}

impl Pgn {
    /// Creates a new PGN with basic information
    pub fn new(fen: String, moves: Vec<String>, result: GameResult) -> Self {
        Self {
            fen,
            moves,
            result,
            tags: Vec::new(),
        }
    }

    /// Adds a tag to the PGN metadata
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.push((key, value));
    }
}

impl std::fmt::Display for Pgn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write tags
        for (key, value) in &self.tags {
            writeln!(f, "[{} \"{}\"]", key, value)?;
        }

        // Write FEN if not starting position
        let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        if self.fen != starting_fen {
            writeln!(f, "[FEN \"{}\"]", self.fen)?;
            writeln!(f, "[SetUp \"1\"]")?;
        }

        // Write result tag
        writeln!(f, "[Result \"{}\"]", self.result)?;
        writeln!(f)?;

        // Write moves in numbered format
        let mut move_text = String::new();
        for (i, mv) in self.moves.iter().enumerate() {
            if i % 2 == 0 {
                // White's move
                move_text.push_str(&format!("{}. {} ", i / 2 + 1, mv));
            } else {
                // Black's move
                move_text.push_str(&format!("{} ", mv));
            }
        }

        // Add result at the end
        move_text.push_str(&format!("{}", self.result));

        // Word wrap at 80 characters
        let mut line = String::new();
        for word in move_text.split_whitespace() {
            if line.len() + word.len() + 1 > 80 {
                writeln!(f, "{}", line)?;
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            writeln!(f, "{}", line)?;
        }

        Ok(())
    }
}

/// Plays a match between two engines from a given FEN position.
///
/// # Arguments
/// * `white_engine` - The engine playing as white
/// * `black_engine` - The engine playing as black
/// * `fen` - The FEN string representing the starting position
/// * `max_moves` - Maximum number of moves before declaring a draw (optional)
///
/// # Returns
/// A tuple containing the game result, final game state, and PGN of the game
///
/// # Example
/// ```
/// use cheese_engine::arbiter::play_match;
/// use cheese_engine::negamax::Negamax;
/// use cheese_engine::evaluation::PstEval;
///
/// let mut white_engine = Negamax::new();
/// let mut black_engine = Negamax::new();
///
/// let (result, final_state, pgn) = play_match::<PstEval>(
///     &mut white_engine,
///     &mut black_engine,
///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
///     Some(100), // Max 100 moves
/// ).unwrap();
///
/// println!("Game result: {}", result);
/// println!("PGN:\n{}", pgn);
/// ```
///
pub fn play_match<E: EvaluateEngine>(
    white_engine: &mut impl SearchEngine<E>,
    black_engine: &mut impl SearchEngine<E>,
    fen: &str,
    max_moves: Option<usize>,
) -> Result<(GameResult, GameState, Pgn), String> {
    // Parse the FEN and create game state
    let board = Board::from_str(fen).map_err(|e| format!("Invalid FEN: {}", e))?;
    let mut state = GameState::from_board(board);

    // Clear search state for both engines
    white_engine.clear_search_state();
    black_engine.clear_search_state();

    let max_moves = max_moves.unwrap_or(200);
    let mut move_count = 0;
    let mut move_list: Vec<String> = Vec::new();

    // Default time control (infinite)
    let time_info = TimeInfo::default();

    // Helper function to create PGN from current state
    let create_pgn = |result: GameResult, moves: Vec<String>| {
        Pgn::new(fen.to_string(), moves, result)
    };

    loop {
        let board = state.last_board();

        // Check for terminal conditions
        match board.status() {
            BoardStatus::Checkmate => {
                // The side to move is checkmated, so the other side wins
                let result = if board.side_to_move() == chess::Color::White {
                    GameResult::BlackWins
                } else {
                    GameResult::WhiteWins
                };
                return Ok((result, state, create_pgn(result, move_list)));
            }
            BoardStatus::Stalemate => {
                return Ok((GameResult::Draw, state, create_pgn(GameResult::Draw, move_list)));
            }
            BoardStatus::Ongoing => {}
        }

        // Check for move limit
        if move_count >= max_moves {
            return Ok((GameResult::Draw, state, create_pgn(GameResult::Draw, move_list)));
        }

        // Select the engine based on side to move
        let best_move = if board.side_to_move() == chess::Color::White {
            white_engine.next_move(state.clone(), time_info.clone())
        } else {
            black_engine.next_move(state.clone(), time_info.clone())
        };

        // Get the next move from the engine
        let best_move = match best_move {
            Some(mv) => mv,
            None => {
                // Engine resigned or couldn't find a move
                let result = if board.side_to_move() == chess::Color::White {
                    GameResult::BlackWins
                } else {
                    GameResult::WhiteWins
                };
                return Ok((result, state, create_pgn(result, move_list)));
            }
        };

        // Convert move to SAN (Standard Algebraic Notation)
        let san_move = move_to_san(&board, best_move);
        move_list.push(san_move);

        // Make the move and check for threefold repetition
        let repetition_count = state.make_move(best_move);
        if repetition_count >= 3 {
            return Ok((GameResult::Draw, state, create_pgn(GameResult::Draw, move_list)));
        }

        move_count += 1;
        log::info!("Ply count: {}", move_count);
    }
}

/// Converts a ChessMove to Standard Algebraic Notation (SAN)
fn move_to_san(board: &Board, mv: ChessMove) -> String {
    use chess::{Piece, MoveGen, Square};

    let piece = board.piece_on(mv.get_source());
    let source = mv.get_source();
    let dest = mv.get_dest();
    let is_capture = board.piece_on(dest).is_some();

    // Check if the move results in check or checkmate
    let new_board = board.make_move_new(mv);
    let gives_check = new_board.checkers().popcnt() > 0;
    let is_checkmate = gives_check && new_board.status() == BoardStatus::Checkmate;

    let mut san = String::new();

    match piece {
        Some(Piece::King) => {
            // Check for castling
            if source == Square::E1 && dest == Square::G1 {
                san.push_str("O-O");
            } else if source == Square::E1 && dest == Square::C1 {
                san.push_str("O-O-O");
            } else if source == Square::E8 && dest == Square::G8 {
                san.push_str("O-O");
            } else if source == Square::E8 && dest == Square::C8 {
                san.push_str("O-O-O");
            } else {
                san.push('K');
                if is_capture {
                    san.push('x');
                }
                san.push_str(&format!("{}", dest));
            }
        }
        Some(Piece::Pawn) => {
            // Pawn moves
            if is_capture {
                san.push((b'a' + source.get_file().to_index() as u8) as char);
                san.push('x');
            }
            san.push_str(&format!("{}", dest));

            // Promotion
            if let Some(promo) = mv.get_promotion() {
                san.push('=');
                san.push(match promo {
                    Piece::Queen => 'Q',
                    Piece::Rook => 'R',
                    Piece::Bishop => 'B',
                    Piece::Knight => 'N',
                    _ => '?',
                });
            }
        }
        Some(p) => {
            // Other pieces (Knight, Bishop, Rook, Queen)
            let piece_char = match p {
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Rook => 'R',
                Piece::Queen => 'Q',
                _ => '?',
            };
            san.push(piece_char);

            // Check for disambiguation
            let same_piece_moves: Vec<ChessMove> = MoveGen::new_legal(board)
                .filter(|&m| {
                    board.piece_on(m.get_source()) == Some(p) &&
                    m.get_dest() == dest &&
                    m.get_source() != source
                })
                .collect();

            if !same_piece_moves.is_empty() {
                let same_file = same_piece_moves.iter().any(|&m| m.get_source().get_file() == source.get_file());
                let same_rank = same_piece_moves.iter().any(|&m| m.get_source().get_rank() == source.get_rank());

                if !same_file {
                    san.push((b'a' + source.get_file().to_index() as u8) as char);
                } else if !same_rank {
                    san.push((b'1' + source.get_rank().to_index() as u8) as char);
                } else {
                    san.push((b'a' + source.get_file().to_index() as u8) as char);
                    san.push((b'1' + source.get_rank().to_index() as u8) as char);
                }
            }

            if is_capture {
                san.push('x');
            }
            san.push_str(&format!("{}", dest));
        }
        None => {
            // Shouldn't happen
            san.push_str(&format!("{}", mv));
        }
    }

    // Add check/checkmate symbols
    if is_checkmate {
        san.push('#');
    } else if gives_check {
        san.push('+');
    }

    san
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::AnyMove;
    use crate::evaluation::CountMaterial;

    #[test]
    fn test_play_from_startpos() {
        let mut white = AnyMove;
        let mut black = AnyMove;

        let result = play_match::<CountMaterial>(
            &mut white,
            &mut black,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            Some(10),
        );

        assert!(result.is_ok());
        let (game_result, _state, pgn) = result.unwrap();
        // With AnyMove engines, it should hit the move limit
        assert_eq!(game_result, GameResult::Draw);
        assert_eq!(pgn.moves.len(), 10);
        println!("PGN:\n{}", pgn);
    }

    #[test]
    fn test_play_from_checkmate_position() {
        let mut white = AnyMove;
        let mut black = AnyMove;

        // Position where black is checkmated
        let result = play_match::<CountMaterial>(
            &mut white,
            &mut black,
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
            Some(100),
        );

        assert!(result.is_ok());
        let (game_result, _state, pgn) = result.unwrap();
        assert_eq!(game_result, GameResult::BlackWins);
        println!("PGN:\n{}", pgn);
    }

    #[test]
    fn test_invalid_fen() {
        let mut white = AnyMove;
        let mut black = AnyMove;

        let result = play_match::<CountMaterial>(
            &mut white,
            &mut black,
            "invalid fen string",
            Some(100),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_pgn_display() {
        let mut pgn = Pgn::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string(), "Nc6".to_string()],
            GameResult::Draw,
        );
        pgn.add_tag("Event".to_string(), "Test Game".to_string());
        pgn.add_tag("White".to_string(), "Engine1".to_string());
        pgn.add_tag("Black".to_string(), "Engine2".to_string());

        let pgn_str = format!("{}", pgn);
        assert!(pgn_str.contains("[Event \"Test Game\"]"));
        assert!(pgn_str.contains("1. e4 e5 2. Nf3 Nc6"));
        assert!(pgn_str.contains("1/2-1/2"));
    }
}
