use std::path::Path;

use colored::Colorize;

use crate::error::{Error, Result};
use crate::git;
use crate::side_repo::SideRepo;
use crate::tracked::TrackedPaths;

/// Remove a path from side tracking.
///
/// # Errors
///
/// Returns an error if the path is not tracked or if unstaging fails.
pub fn run(path: &Path) -> Result<()> {
    let relative_path = git::repo_relative(path)?;

    // Open side repo
    let repo = SideRepo::open()?;

    if !repo.is_initialized() {
        return Err(Error::PathNotTracked(relative_path));
    }

    // Load tracked paths
    let mut tracked = TrackedPaths::load(&repo)?;

    // Check if tracked
    if !tracked.contains(&relative_path) {
        return Err(Error::PathNotTracked(relative_path));
    }

    // Remove from tracked list
    tracked.remove(&relative_path);
    tracked.save(&repo)?;

    // A path still inside another tracked directory stays versioned: only
    // the redundant list entry goes away.
    if let Some(parent) = tracked.covering(&relative_path) {
        println!(
            "{} {} (still tracked via {})",
            "Removed entry:".yellow().bold(),
            relative_path.display(),
            parent.display()
        );
        return Ok(());
    }

    // Unstage from side repo
    repo.unstage(&relative_path)?;

    println!(
        "{} {}",
        "Untracked:".yellow().bold(),
        relative_path.display()
    );

    Ok(())
}
