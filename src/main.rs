mod arbiter;
mod engine;
mod evaluation;
mod mcts;
mod negamax;
mod uci;

use fern;
use log;

use crate::engine::{GameState, SearchEngine};
use crate::evaluation::{CountMaterial, PstEval};
use crate::negamax::Negamax;
use crate::uci::uci_loop;

use crate::arbiter::play_match;

fn run_match() {
    let mut white_engine = Negamax::new();
    let mut black_engine = Negamax::new();

    let (result, _final_state, pgn) = play_match::<PstEval>(
        &mut white_engine,
        &mut black_engine,
        "r2q1rk1/pp2bppp/2np1n2/2p5/2P1P3/2N2N2/PP1B1PPP/R2Q1RK1 w - - 0 10",
        Some(200),
    )
    .unwrap();

    println!("Game result: {}", result);
    println!("PGN:\n{}", pgn);
}
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
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr()) // log to console in STDERR
        .chain(fern::log_file(
            "/home/abdel/Documents/cheese-engine/logfile.txt",
        )?) // log to file, useful when the engine is called by another program and I can't read STDERR directly.
        .apply()?;

    run_match();
    
    // log::info!("Starting UCI loop");

    let mut engine = Negamax::new();
    // uci_loop::<PstEval, _>(&mut engine);
    
    Ok(())
}
