//! Entry point for the standalone `pumpkin-gui` binary.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use clap::Parser;

/// Optional Qt6 monitoring and console window for the Pumpkin server.
#[derive(Parser)]
struct Args {
    #[arg(long)]
    connect: Option<String>,
}

fn main() {
    let args = Args::parse();

    let Some(endpoint) = args
        .connect
        .or_else(|| std::env::var(pumpkin_gui_api::GUI_ENDPOINT_ENV).ok())
    else {
        eprintln!(
            "pumpkin-gui: no server endpoint given (pass --connect or set {})",
            pumpkin_gui_api::GUI_ENDPOINT_ENV
        );
        std::process::exit(1);
    };

    let client = match pumpkin_gui::client::connect(&endpoint) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("pumpkin-gui: could not connect to {endpoint}: {err}");
            std::process::exit(1);
        }
    };

    match pumpkin_gui::run(client) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("pumpkin-gui: {err}");
            std::process::exit(1);
        }
    }
}
