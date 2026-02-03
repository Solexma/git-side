use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;

use git_side::commands;

#[derive(Parser)]
#[command(
    name = "git-side",
    about = "Version files that should not live in the main repo",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Track a file or directory (forced, bypasses gitignore)
    Add {
        /// Path to track
        path: PathBuf,
    },

    /// Untrack a path from side repo
    Rm {
        /// Path to untrack
        path: PathBuf,
    },

    /// Show side repo status
    Status,

    /// Commit staged changes to side repo
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },

    /// Show side repo history
    Log {
        /// Additional arguments to pass to git log
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Sync side-tracked paths and commit using last main repo message
    Auto,

    /// Initialize side repo with optional custom path
    Init {
        /// Custom base path for side repo storage
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Manage git hooks for auto-sync
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand)]
enum HookAction {
    /// Install git hook to run auto on commits
    Install {
        /// Hook to install (default: post-commit)
        #[arg(long, default_value = "post-commit")]
        on: String,
    },

    /// Remove git hook
    Uninstall {
        /// Hook to remove (default: post-commit)
        #[arg(long, default_value = "post-commit")]
        on: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add { path } => commands::add::run(&path),
        Commands::Rm { path } => commands::rm::run(&path),
        Commands::Status => commands::status::run(),
        Commands::Commit { message } => commands::commit::run(&message),
        Commands::Log { args } => commands::log::run(&args),
        Commands::Auto => commands::auto::run(),
        Commands::Init { path } => commands::init::run(path.as_deref()),
        Commands::Hook { action } => match action {
            HookAction::Install { on } => commands::hook::install(&on),
            HookAction::Uninstall { on } => commands::hook::uninstall(&on),
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            ExitCode::FAILURE
        }
    }
}
