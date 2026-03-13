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
    /// Start the development server with file watching
    Dev {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    // Phase 3+: Build, Deploy, Add, Explain, Validate, Migrate
}
