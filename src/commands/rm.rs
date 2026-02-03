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
    let work_tree = git::repo_root()?;

    // Normalize path: make it relative to work tree
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&work_tree)
            .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
    } else {
        path.to_path_buf()
    };

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
    tracked.save()?;

    // Unstage from side repo
    repo.unstage(&relative_path)?;

    println!(
        "{} {}",
        "Untracked:".yellow().bold(),
        relative_path.display()
    );

    Ok(())
}
