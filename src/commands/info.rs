use colored::Colorize;

use crate::error::Result;
use crate::side_repo::SideRepo;
use crate::tracked::TrackedPaths;

/// Show info about git-side.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened.
pub fn run() -> Result<()> {
    println!("{}", "git-side".bold());
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A Git subcommand that versions files and directories that");
    println!("should not live in the main repo, using a per-project bare repo.");
    println!();
    println!("{}", "Author:".cyan());
    println!("  MiPnamic <mipnamic@mipnamic.net>");
    println!("  https://github.com/MiPnamic");
    println!();
    println!("{}", "Project:".cyan());
    println!("  https://github.com/Solexma/git-side");
    println!("  MIT License - Solexma LLC");
    println!();

    // Show project-specific info if in a git repo
    if let Ok(repo) = SideRepo::open() {
        println!("{}", "Current project:".cyan());
        println!("  Root SHA: {}", repo.root_sha);
        println!("  Side repo: {}", repo.git_dir.display());
        println!("  Initialized: {}", if repo.is_initialized() { "yes".green() } else { "no".yellow() });

        if repo.is_initialized() {
            if let Ok(tracked) = TrackedPaths::load(&repo) {
                let paths: Vec<_> = tracked.paths().iter().collect();
                if paths.is_empty() {
                    println!("  Tracked paths: {}", "none".yellow());
                } else {
                    println!("  Tracked paths:");
                    for path in paths {
                        println!("    - {}", path.display());
                    }
                }
            }
        }
    }

    Ok(())
}
