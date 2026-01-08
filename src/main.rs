mod engine;
mod evaluation;
mod mcts;
mod negamax;
mod uci;

use fern;
use log;

use crate::engine::AnyMove;
use crate::evaluation::CountMaterial;
use crate::uci::uci_loop;

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
            "/home/abdel/Documents/cheese-engine/logfile.txt",
        )?) // log to file, useful when the engine is called by another program and I can't read STDERR directly.
        .apply()?;

    log::info!("Starting UCI loop");
    let mut engine = AnyMove;
    uci_loop::<CountMaterial, _>(&mut engine);
    Ok(())
}
