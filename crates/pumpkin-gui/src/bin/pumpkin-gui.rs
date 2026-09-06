//! Entry point for the standalone `pumpkin-gui` binary.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use clap::Parser;

/// Optional Qt6 monitoring and console window for the Pumpkin server.
///
/// Run with no arguments to find or start a server automatically
/// (see `gui.conf`, created next to this executable on first run).
#[derive(Parser)]
struct Args {
    /// Attaches to a server already running at this endpoint
    #[arg(long)]
    attach: Option<String>,
}

fn main() {
    let args = Args::parse();

    match pumpkin_gui::run(args.attach) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("pumpkin-gui: {err}");
            std::process::exit(1);
        }
    }
}
