use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Run a git command and return stdout on success.
///
/// # Errors
///
/// Returns an error if the git command fails to execute or exits with non-zero status.
pub fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to execute git: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(Error::GitCommandFailed(stderr))
    }
}

/// Run a git command with a specific work-tree and git-dir.
///
/// # Errors
///
/// Returns an error if the git command fails to execute or exits with non-zero status.
pub fn run_with_paths(git_dir: &Path, work_tree: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(work_tree)
        .env("GIT_DIR", git_dir)
        .env("GIT_WORK_TREE", work_tree)
        // Clear any inherited git environment from hooks
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .args(args)
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to execute git: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(Error::GitCommandFailed(stderr))
    }
}

/// Check if we're inside a git repository.
#[must_use]
pub fn is_in_repo() -> bool {
    run(&["rev-parse", "--is-inside-work-tree"])
        .is_ok_and(|s| s == "true")
}

/// Get the root directory of the current git repository.
///
/// # Errors
///
/// Returns an error if not inside a git repository.
pub fn repo_root() -> Result<PathBuf> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let root = run(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(root))
}

/// Get the hooks directory git actually runs hooks from.
///
/// Resolves to the common dir in linked worktrees and honours `core.hooksPath`.
///
/// # Errors
///
/// Returns an error if not inside a git repository.
pub fn hooks_dir() -> Result<PathBuf> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let dir = run(&["rev-parse", "--git-path", "hooks"])?;
    Ok(PathBuf::from(dir))
}

/// Get the initial commit SHA of the repository (project identifier).
///
/// # Errors
///
/// Returns an error if not inside a git repository or if the repository has no commits.
pub fn initial_commit_sha() -> Result<String> {
    if !is_in_repo() {
        return Err(Error::NotInGitRepo);
    }
    let sha = run(&["rev-list", "--max-parents=0", "HEAD"])
        .map_err(|_| Error::NoCommits)?;

    // Take first line if multiple roots exist (rare but possible)
    Ok(sha.lines().next().unwrap_or(&sha).to_string())
}

/// Get the last commit message from the main repository.
///
/// # Errors
///
/// Returns an error if the git log command fails.
pub fn last_commit_message() -> Result<String> {
    run(&["log", "-1", "--format=%B"])
}

/// Git dir of the linked worktree we are in (`<common>/worktrees/<name>`),
/// or `None` in the main worktree.
///
/// # Errors
///
/// Returns an error if not inside a git repository.
pub fn linked_worktree_dir() -> Result<Option<PathBuf>> {
    let out = run(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-dir",
        "--git-common-dir",
    ])?;
    let mut lines = out.lines();
    let (Some(git_dir), Some(common_dir)) = (lines.next(), lines.next()) else {
        return Err(Error::GitCommandFailed(format!("unexpected rev-parse output: {out}")));
    };
    if git_dir == common_dir {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(git_dir)))
}

/// Resolve a user-supplied path (relative to the current directory, or
/// absolute) to a path relative to the repository root, the form used in
/// the tracked list. `.` and `..` are resolved lexically.
///
/// # Errors
///
/// Returns an error if not inside a git repository, or if the path is
/// outside the repository or is the repository root itself.
pub fn repo_relative(path: &Path) -> Result<PathBuf> {
    let root = repo_root()?;
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Relative to the root via the prefix, so a symlinked cwd still matches.
        root.join(run(&["rev-parse", "--show-prefix"])?).join(path)
    };

    let normalized = normalize(&joined);
    let relative = normalized
        .strip_prefix(&root)
        .map(Path::to_path_buf)
        .or_else(|_| {
            // An absolute path reaching the repo through a symlink.
            normalized
                .canonicalize()
                .ok()
                .and_then(|canonical| canonical.strip_prefix(&root).ok().map(Path::to_path_buf))
                .ok_or_else(|| Error::PathOutsideRepo(path.to_path_buf()))
        })?;

    if relative.as_os_str().is_empty() {
        return Err(Error::PathIsRepoRoot);
    }
    Ok(relative)
}

/// Resolve `.` and `..` components without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}
