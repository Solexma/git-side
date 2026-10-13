use colored::Colorize;

use crate::error::{Error, Result};
use crate::side_repo::SideRepo;

/// Check out side files missing from the work-tree.
///
/// In a linked worktree this also sets up its side state on first use.
/// Existing files are never overwritten.
///
/// # Errors
///
/// Returns an error if there is no side repo or if git commands fail.
pub fn run() -> Result<()> {
    let repo = SideRepo::open()?;
    if !repo.has_side_repo() {
        return Err(Error::NoSideRepo);
    }
    repo.ensure_initialized()?;

    let written = repo.materialize()?;
    if written.is_empty() {
        println!("{}", "Side files already present.".green());
    } else {
        for file in &written {
            println!("{} {file}", "Checked out:".green().bold());
        }
    }
    Ok(())
}
