use std::path::Path;

use colored::Colorize;

use crate::config;
use crate::error::Result;
use crate::git;

/// Initialize side repo with optional custom path.
///
/// # Errors
///
/// Returns an error if not in a git repo or if config cannot be written.
pub fn run(path: Option<&Path>) -> Result<()> {
    // Get the project identifier
    let work_tree = git::repo_root()?;
    let path_hash = config::hash_path(&work_tree);

    // Get or resolve root SHA
    let root_sha = if let Some(sha) = config::cache_lookup(&path_hash)? {
        sha
    } else {
        let sha = git::initial_commit_sha()?;
        config::cache_store(&path_hash, &sha)?;
        sha
    };

    // Store custom path if provided
    if let Some(base_path) = path {
        config::paths_store(&root_sha, base_path)?;

        println!(
            "{} Side repo will be stored at: {}",
            "Initialized.".green().bold(),
            base_path.join(&root_sha).display()
        );
    } else {
        let default_path = config::default_base_path().join(&root_sha);
        println!(
            "{} Side repo will be stored at: {}",
            "Initialized.".green().bold(),
            default_path.display()
        );
    }

    Ok(())
}
