mod cli;
mod commands;
mod error;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use output::{human, json, OutputMode};

fn main() {
    let cli = Cli::parse();
    let mode = if cli.json {
        OutputMode::Json
    } else {
        OutputMode::Human
    };

    let result = match &cli.command {
        Commands::New { name } => commands::new::run(name, mode),
        Commands::Info => commands::info::run(mode),
    };

    if let Err(e) = result {
        let rotiv_err = e.to_rotiv_error();
        match mode {
            OutputMode::Json => json::print_error(&rotiv_err),
            OutputMode::Human => human::print_error(&rotiv_err),
        }
        std::process::exit(1);
    }
}
