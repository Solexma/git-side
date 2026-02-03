use colored::Colorize;

use crate::error::Result;
use crate::side_repo::SideRepo;

/// Pull side repo from remote.
/// Uses fetch + reset to avoid conflicts — remote always wins.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or pull fails.
pub fn run() -> Result<()> {
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    // Fetch from origin
    repo.git(&["fetch", "origin"])?;

    // Reset to origin/main (remote wins, no conflicts)
    repo.git(&["reset", "--hard", "origin/main"])?;

    println!("{}", "Pulled from remote.".green().bold());
    Ok(())
}
