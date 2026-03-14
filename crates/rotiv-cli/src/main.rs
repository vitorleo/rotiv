mod cli;
mod commands;
mod error;
mod output;

use clap::Parser;
use cli::{AddSubcommand, Cli, Commands};
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
        Commands::Dev { port, host } => commands::dev::run(*port, host, mode),
        Commands::Build { out, minify } => commands::build::run(out.clone(), *minify, mode),
        Commands::Migrate { generate_only, check } => {
            commands::migrate::run(*generate_only, *check, mode)
        }
        Commands::Add(args) => match &args.subcommand {
            AddSubcommand::Route { path } => commands::add::run_add_route(path, mode),
            AddSubcommand::Model { name } => commands::add::run_add_model(name, mode),
            AddSubcommand::Module { name } => commands::add::run_add_module(name, mode),
        },
        Commands::SpecSync => commands::spec_sync::run(mode),
        Commands::Validate { fix } => commands::validate::run(*fix, mode),
        Commands::Explain { topic } => commands::explain::run(topic, mode),
        Commands::ContextRegen => commands::context::run(mode),
        Commands::DiffImpact { file } => commands::diff_impact::run(file, mode),
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
