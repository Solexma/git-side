use colored::Colorize;

use crate::error::Result;
use crate::side_repo::SideRepo;

/// Commit staged changes to side repo.
///
/// # Errors
///
/// Returns an error if there's nothing to commit or if the commit fails.
pub fn run(message: &str) -> Result<()> {
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    // Always stage .side-tracked to ensure it's included
    repo.stage_tracked_file()?;

    repo.commit(message)?;

    println!("{}", "Committed to side repo.".green().bold());
    Ok(())
}
