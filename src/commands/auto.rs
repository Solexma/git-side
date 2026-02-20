use colored::Colorize;

use crate::error::{Error, Result};
use crate::git;
use crate::side_repo::SideRepo;
use crate::tracked::TrackedPaths;

/// Auto-commit: sync all tracked paths using the last main repo commit message.
///
/// # Errors
///
/// Returns an error if no paths are tracked, staging fails, or commit fails.
pub fn run() -> Result<()> {
    let repo = SideRepo::open()?;

    if !repo.is_initialized() {
        return Err(Error::NoTrackedPaths);
    }

    // Load tracked paths
    let tracked = TrackedPaths::load(&repo)?;

    if tracked.is_empty() {
        return Err(Error::NoTrackedPaths);
    }

    // Expand directories to files
    let files = tracked.expand(&repo.work_tree);

    // Get the raw tracked paths for staging (we stage the tracked paths, not expanded files)
    let tracked_paths: Vec<_> = tracked.paths().iter().cloned().collect();

    // Two-pass staging:
    // Pass 1: update tracked files (modifications + deletions) — errors ignored
    repo.stage_update(&tracked_paths);

    // Pass 2: add new files
    repo.stage_new(&tracked_paths)?;

    // Stage .side-tracked file itself (self-aware versioning)
    repo.stage_tracked_file()?;

    // Get last commit message from main repo
    let message = git::last_commit_message()?;

    if message.trim().is_empty() {
        return Err(Error::GitCommandFailed(
            "no commit message found in main repo".to_string(),
        ));
    }

    let prefix = "[git-side]".dimmed();

    // Commit (will return NothingToCommit if nothing changed)
    match repo.commit(&message) {
        Ok(()) => {
            println!(
                "{} {} {}",
                prefix,
                "Auto-committed:".green().bold(),
                message.lines().next().unwrap_or(&message)
            );
            println!(
                "{} {} file(s) synced",
                prefix,
                files.len().to_string().cyan()
            );
            // Try to push — silently ignored if no remote
            if repo.push().is_ok() {
                println!("{} {}", prefix, "Pushed to remote.".green().bold());
            }
        }
        Err(Error::NothingToCommit) => {
            println!("{} {}", prefix, "Nothing to commit (side repo is up to date).".yellow());
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
