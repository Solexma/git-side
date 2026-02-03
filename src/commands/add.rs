use std::path::Path;

use colored::Colorize;

use crate::error::{Error, Result};
use crate::git;
use crate::side_repo::SideRepo;
use crate::tracked::TrackedPaths;

/// Add a path to side tracking.
///
/// # Errors
///
/// Returns an error if the path doesn't exist, is already tracked, or if staging fails.
pub fn run(path: &Path) -> Result<()> {
    let work_tree = git::repo_root()?;

    // Normalize path: make it relative to work tree
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&work_tree)
            .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
    } else {
        path.to_path_buf()
    };

    // Check if path exists
    let full_path = work_tree.join(&relative_path);
    if !full_path.exists() {
        return Err(Error::PathNotFound(relative_path));
    }

    // Open side repo (lazy init)
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    // Load tracked paths
    let mut tracked = TrackedPaths::load(&repo)?;

    // Check if already tracked
    if tracked.contains(&relative_path) {
        return Err(Error::PathAlreadyTracked(relative_path));
    }

    // Add to tracked list
    tracked.add(&relative_path);
    tracked.save()?;

    // Stage the path
    repo.stage(&relative_path)?;

    println!(
        "{} {}",
        "Tracking:".green().bold(),
        relative_path.display()
    );

    Ok(())
}
