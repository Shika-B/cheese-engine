use crate::engine::{EvaluateEngine, GameState, SearchEngine, TimeInfo};

use chess::{Board, ChessMove, Piece, Square};
use log::{debug, error};
use vampirc_uci::{UciFen, UciMessage, UciMove, UciPiece, UciSquare, UciTimeControl, parse_one};

use std::{
    io::{self, BufRead},
    str::FromStr,
};

pub fn uci_loop<E: EvaluateEngine, S: SearchEngine<E>>(engine: &mut S) -> () {
    let stdin = io::stdin();

    let mut game_state = GameState::default();

    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        let uci_message = parse_one(&line);
        log::debug!("Received:{}", line);
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
                fen,
                moves,
            } => {
                // Clear engine search state when setting a new position
                engine.clear_search_state();

                if let Some(UciFen(fen_str)) = fen {
                    let board = Board::from_str(&fen_str).expect("Failed to build board from FEN");
                    game_state = GameState::from_board(board);
                } else if startpos {
                    game_state = GameState::default();
                } else {
                    error!("Position command with neither FEN nor startpos!");
                }

                // Apply moves regardless of whether we started from FEN or startpos
                for mv in moves {
                    let chess_mv = from_uci_move(mv);
                    game_state.make_move(chess_mv);
                }
            }
            UciMessage::Go {
                time_control,
                search_control: _,
            } => {
                // TODO: Implement search control parsing ?
                let time_control = if let Some(tc) = time_control {
                    match tc {
                        UciTimeControl::Infinite => None,
                        UciTimeControl::TimeLeft {
                            white_time,
                            black_time,
                            white_increment,
                            black_increment,
                            moves_to_go,
                        } => Some(TimeInfo {
                            white_time,
                            black_time,
                            white_increment,
                            black_increment,
                            moves_to_go,
                            move_time: None,
                        }),
                        UciTimeControl::MoveTime(move_time) => Some(TimeInfo {
                            move_time: Some(move_time),
                            white_time: None,
                            black_time: None,
                            white_increment: None,
                            black_increment: None,
                            moves_to_go: None,
                        }),
                        UciTimeControl::Ponder => None,
                    }
                } else {
                    None
                };

                let best_move = engine.next_move(game_state.clone(), &time_control);
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
                    None => {
                        log::info!("Resigning");
                        println!("bestmove 0000") // Resigns
                    }
                }
            }
            UciMessage::Unknown(message, _) => {
                log::error!("Warning, unknown string {}", message)
            }
            m => {
                log::error!("Unimplemented {:#?}", m);
            }
        }
    }
}

// Two conversion functions

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
