use colored::Colorize;

use crate::error::{Error, Result};
use crate::side_repo::SideRepo;

/// What `git side merge` should do.
pub enum Action<'a> {
    /// Merge the side branch of worktree `name` (or `main`) into this one.
    Start(&'a str),
    /// Stage resolved conflicts and create the merge commit.
    Continue,
    /// Abort an in-progress merge.
    Abort,
}

/// Merge side branches between worktrees.
///
/// # Errors
///
/// Returns an error if the branch does not exist, the merge conflicts, or
/// git commands fail.
pub fn run(action: &Action) -> Result<()> {
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    match action {
        Action::Start(name) => start(&repo, name),
        Action::Continue => {
            if !repo.merge_in_progress() {
                return Err(Error::NoMergeInProgress);
            }
            let conflicted = conflicted_files(&repo)?;
            if !conflicted.is_empty() {
                let mut args = vec!["add", "-A", "-f", "--"];
                args.extend(conflicted.iter().map(String::as_str));
                repo.git(&args)?;
            }
            repo.git(&["commit", "--no-edit"])?;
            repo.sync_tracked_file()?;
            println!("{}", "Merge completed.".green().bold());
            Ok(())
        }
        Action::Abort => {
            if !repo.merge_in_progress() {
                return Err(Error::NoMergeInProgress);
            }
            repo.git(&["merge", "--abort"])?;
            repo.sync_tracked_file()?;
            println!("{}", "Merge aborted.".yellow().bold());
            Ok(())
        }
    }
}

fn start(repo: &SideRepo, name: &str) -> Result<()> {
    let current = repo.worktree.as_deref().unwrap_or("main");
    if name == current {
        return Err(Error::MergeIntoSelf(name.to_string()));
    }
    let branch = if name == "main" {
        "main".to_string()
    } else {
        format!("wt/{name}")
    };
    if !repo.ref_exists(&format!("refs/heads/{branch}")) {
        return Err(Error::UnknownSideBranch(name.to_string()));
    }

    let result = repo.git(&["merge", "--no-edit", "--allow-unrelated-histories", &branch]);
    // The merged path list is in the index (merge=union never conflicts).
    repo.sync_tracked_file()?;

    match result {
        Ok(_) => {
            println!("{} {branch}", "Merged:".green().bold());
            Ok(())
        }
        Err(e) => {
            let conflicted = conflicted_files(repo)?;
            if conflicted.is_empty() {
                Err(e)
            } else {
                Err(Error::MergeConflict(conflicted.join(", ")))
            }
        }
    }
}

fn conflicted_files(repo: &SideRepo) -> Result<Vec<String>> {
    let out = repo.git(&["diff", "-z", "--name-only", "--diff-filter=U"])?;
    Ok(out
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect())
}
