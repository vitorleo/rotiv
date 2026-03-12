use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rotiv", version, about = "Rotiv — AI-native full-stack framework CLI")]
pub struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Rotiv project
    New {
        /// Project name (also used as directory name)
        name: String,
    },
    /// Print framework version and project spec summary
    Info,
    // Phase 2+: Dev, Build, Deploy, Add, Explain, Validate, Migrate
}
