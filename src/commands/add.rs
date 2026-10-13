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

    let relative_path = git::repo_relative(path)?;

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
    let covered = tracked.covering(&relative_path).is_some();
    if !covered {
        tracked.add(&relative_path);
        tracked.save(&repo)?;
    }

    // Stage the path
    repo.stage(&relative_path)?;

    let label = if covered { "Staged:" } else { "Tracking:" };
    println!("{} {}", label.green().bold(), relative_path.display());

    Ok(())
}
