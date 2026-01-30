mod arbiter;
mod engine;
mod mcts;
mod negamax;
mod evaluation;
mod uci;

use fern;
use log;

use crate::evaluation::{PstEval};
use crate::mcts::{MCTS, MCTSEngine};
use crate::negamax::Negamax;
use crate::uci::uci_loop;
use chess::{Board, Square};
use crate::evaluation::{NnueEval};

// // fn run_match() {
// //     let mut white_engine = Negamax::new();
// //     let mut black_engine = Negamax::new();

// //     let (result, _final_state, pgn) = play_match::<PstEval>(
// //         &mut white_engine,
// //         &mut black_engine,
// //         "r2q1rk1/pp2bppp/2np1n2/2p5/2P1P3/2N2N2/PP1B1PPP/R2Q1RK1 w - - 0 10",
// //         Some(200),
// //     )
// //     .unwrap();

// //     println!("Game result: {}", result);
// //     println!("PGN:\n{}", pgn);
// // }
fn main() -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stderr()) // log to console in STDERR
        .chain(fern::log_file(
            "./logfile.txt",
        )?) // log to file, useful when the engine is called by another program and I can't read STDERR directly.
        .apply()?;



    log::info!("Starting UCI loop");
    let eval = PstEval;
    let mut engine = Negamax::new(eval);
    uci_loop::<PstEval, _>(&mut engine);
    
    Ok(())
}
