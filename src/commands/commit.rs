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
    repo.commit(message)?;

    println!("{}", "Committed to side repo.".green().bold());
    Ok(())
}
