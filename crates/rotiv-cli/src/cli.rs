use clap::{Parser, Subcommand, Args};

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
    /// Compile the project to dist/
    Build {
        /// Output directory (defaults to <project>/dist)
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        /// Enable minification
        #[arg(long)]
        minify: bool,
    },
    /// Run database migrations (generate, apply, or check)
    Migrate {
        /// Generate migration files without applying them
        #[arg(long)]
        generate_only: bool,
        /// Check for pending migrations without applying
        #[arg(long)]
        check: bool,
    },
    /// Scaffold a new route or model file with annotated comments
    Add(AddArgs),
    /// Sync .rotiv/spec.json with current filesystem state
    SpecSync,
    /// Run static analysis against framework rules
    Validate {
        /// Apply auto-fixes for fixable diagnostics
        #[arg(long)]
        fix: bool,
    },
    /// Query the built-in knowledge base
    Explain {
        /// Topic name (routes, models, loader, action, middleware, signals, migrate, context, modules, deploy)
        topic: String,
    },
    /// Regenerate .rotiv/context.md from current project state
    ContextRegen,
    /// Analyze which routes are affected by changes to a file
    DiffImpact {
        /// File to analyze (e.g. app/models/user.ts)
        file: String,
    },
    /// Deploy the project to a remote server via SSH
    Deploy {
        /// Remote host (overrides .rotiv/deploy.json)
        #[arg(long)]
        host: Option<String>,
        /// Remote user (default: root)
        #[arg(long)]
        user: Option<String>,
        /// Remote path to deploy to (overrides .rotiv/deploy.json)
        #[arg(long, name = "path")]
        remote_path: Option<String>,
        /// Systemd service name to restart (overrides .rotiv/deploy.json)
        #[arg(long)]
        service: Option<String>,
        /// Create .rotiv/deploy.json config template and exit
        #[arg(long)]
        init: bool,
        /// Print deployment steps without executing them
        #[arg(long)]
        dry_run: bool,
        /// Skip the build step (use existing dist/)
        #[arg(long)]
        skip_build: bool,
    },
}

#[derive(Args)]
pub struct AddArgs {
    #[command(subcommand)]
    pub subcommand: AddSubcommand,
}

#[derive(Subcommand)]
pub enum AddSubcommand {
    /// Generate a route file at app/routes/<path>.tsx
    Route {
        /// Route path, e.g. "users/[id]" or "products"
        path: String,
    },
    /// Generate a model file at app/models/<name>.ts
    Model {
        /// Model name in PascalCase, e.g. "Post" or "UserProfile"
        name: String,
    },
    /// Scaffold a module directory at app/modules/<name>/
    Module {
        /// Module name in lowercase-hyphen format, e.g. "auth" or "file-uploads"
        name: String,
    },
}
