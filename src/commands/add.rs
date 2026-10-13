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
/// Returns an error if the path doesn't exist or if staging fails.
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

    // Like `git add`: a path that is already tracked (or inside a tracked
    // directory) just gets its current content staged.
    let covered = tracked.paths().iter().any(|t| relative_path.starts_with(t));
    if !covered {
        tracked.add(&relative_path);
        tracked.save()?;
    }

    // Stage the path
    repo.stage(&relative_path)?;

    // Stage .side-tracked file itself
    repo.stage_tracked_file()?;

    let label = if covered { "Staged:" } else { "Tracking:" };
    println!("{} {}", label.green().bold(), relative_path.display());

    Ok(())
}
