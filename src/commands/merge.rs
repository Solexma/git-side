use colored::Colorize;

use crate::error::{Error, Result};
use crate::side_repo::SideRepo;

/// What `git side merge` should do.
pub enum Action<'a> {
    /// Merge a side branch into the current one. `main` and `wt/<name>` are
    /// branch names; anything else is shorthand for `wt/<name>`.
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
    if !repo.has_side_repo() {
        return Err(Error::NoSideRepo);
    }
    repo.ensure_initialized()?;

    match action {
        Action::Start(target) => start(&repo, target),
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
            println!("{}", "Merge completed.".green().bold());
            Ok(())
        }
        Action::Abort => {
            if !repo.merge_in_progress() {
                return Err(Error::NoMergeInProgress);
            }
            repo.git(&["merge", "--abort"])?;
            println!("{}", "Merge aborted.".yellow().bold());
            Ok(())
        }
    }
}

fn start(repo: &SideRepo, target: &str) -> Result<()> {
    let branch = if target == "main" || target.starts_with("wt/") {
        target.to_string()
    } else {
        format!("wt/{target}")
    };
    if branch == repo.branch() {
        return Err(Error::MergeIntoSelf(branch));
    }
    if !repo.ref_exists(&format!("refs/heads/{branch}")) {
        return Err(Error::UnknownSideBranch(target.to_string()));
    }

    // The tracked list merges with merge=union, so it never conflicts.
    match repo.git(&["merge", "--no-edit", "--allow-unrelated-histories", &branch]) {
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
