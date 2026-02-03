use colored::Colorize;

use crate::error::Result;
use crate::side_repo::SideRepo;

/// Push side repo to remote.
/// Uses force push — local always wins, no conflicts.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or push fails.
pub fn run() -> Result<()> {
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    // Force push to origin main — local wins, no questions asked
    repo.git(&["push", "-u", "--force", "origin", "main"])?;

    println!("{}", "Pushed to remote.".green().bold());
    Ok(())
}
