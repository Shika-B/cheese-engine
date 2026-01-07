use crate::engine::{Engine, GameState};


use vampirc_uci::{UciMessage, UciMove, UciPiece, UciSquare, parse_one};
use chess::{Board, ChessMove, Piece, Square};

use std::io::{self, BufRead};


pub fn uci_loop<T: Engine>(engine: &mut T) -> () {
    let stdin = io::stdin();

    let mut game_state = GameState::default();
    let mut board = Board::default();

    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        let uci_message = parse_one(&line);
        log::debug!("Received: {:#?}", uci_message);
        match uci_message {
            UciMessage::Uci => {
                let name = UciMessage::id_name("Cheese");
                let author = UciMessage::id_author("Shika");

                println!("{}", name);
                println!("{}", author);
                println!("{}", UciMessage::UciOk);
            }
            UciMessage::IsReady => println!("{}", UciMessage::ReadyOk),

            UciMessage::Position {
                startpos,
                fen: _,
                moves,
            } => {
                if !startpos {
                    // TODO: Implement FEN string parsing.
                    unimplemented!("Does not handle FEN string parsing yet")
                }
                game_state = GameState::default();
                board = Board::default();

                for mv in moves {
                    let chess_mv = from_uci_move(mv);
                    board = game_state.update_from_move(board, chess_mv);
                }
            }
            UciMessage::Go {
                time_control: _,
                search_control: _,
            } => {
                // TODO: Implement time control (and search control ?) parsing
                let best_move = engine.next_move(&board, &game_state, None);
                log::debug!("Found move {:#?}", best_move);
                match best_move {
                    Some(mv) => {
                        let uci_move = into_uci_move(mv);
                        let best_move = UciMessage::BestMove {
                            best_move: uci_move,
                            ponder: None,
                        };
                        log::debug!("{}", best_move);
                        println!("{}", best_move);
                    }
                    None => println!("bestmove 0000"), // Resigns
                }
            }
            UciMessage::Unknown(message, _) => {
                log::error!("Warning, unknown string {}", message)
            }
            m => {
                log::error!("Unimplemented {:#?}", m);
                // panic!("Unimplemented {:#?}", m);
            }
        }
    }
}



fn from_uci_move(mv: UciMove) -> ChessMove {
    fn parse_uci_square(sq: UciSquare) -> Square {
        let file_idx = (sq.file as u8 - b'a') as usize;
        let rank_idx = (sq.rank - 1) as usize;
        Square::make_square(chess::ALL_RANKS[rank_idx], chess::ALL_FILES[file_idx])
    }

    let promotion = if let Some(uci_piece) = mv.promotion {
        Some(match uci_piece {
            UciPiece::Bishop => Piece::Bishop,
            UciPiece::Rook => Piece::Rook,
            UciPiece::Knight => Piece::Knight,
            UciPiece::Queen => Piece::Queen,
            _ => unreachable!(),
        })
    } else {
        None
    };
    ChessMove::new(
        parse_uci_square(mv.from),
        parse_uci_square(mv.to),
        promotion,
    )
}

fn into_uci_move(mv: ChessMove) -> UciMove {
    fn into_uci_square(sq: Square) -> UciSquare {
        let file = (b'a' + sq.get_file() as u8) as char;
        let rank = sq.get_rank() as u8 + 1;
        UciSquare { file, rank }
    }

    let promotion = if let Some(piece) = mv.get_promotion() {
        Some(match piece {
            Piece::Bishop => UciPiece::Bishop,
            Piece::Rook => UciPiece::Rook,
            Piece::Knight => UciPiece::Knight,
            Piece::Queen => UciPiece::Queen,
            _ => unreachable!(),
        })
    } else {
        None
    };

    UciMove {
        from: into_uci_square(mv.get_source()),
        to: into_uci_square(mv.get_dest()),
        promotion,
    }
}
