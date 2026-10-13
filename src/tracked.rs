use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::side_repo::SideRepo;

/// Manages the list of tracked paths (`.side-tracked`).
///
/// The list lives in the side repo's index; there is no other copy to keep
/// in sync after pull, merge or worktree setup.
pub struct TrackedPaths {
    paths: BTreeSet<PathBuf>,
}

impl TrackedPaths {
    /// Load tracked paths from the side repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the list cannot be read.
    pub fn load(repo: &SideRepo) -> Result<Self> {
        let paths = repo
            .read_tracked_list()?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(PathBuf::from)
            .collect();
        Ok(Self { paths })
    }

    /// Save tracked paths to the side repo's index.
    ///
    /// # Errors
    ///
    /// Returns an error if the list cannot be written or staged.
    pub fn save(&self, repo: &SideRepo) -> Result<()> {
        let content: String = self
            .paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        repo.write_tracked_list(&content)
    }

    /// Add a path to track.
    pub fn add(&mut self, path: &Path) -> bool {
        self.paths.insert(path.to_path_buf())
    }

    /// Remove a path from tracking.
    pub fn remove(&mut self, path: &Path) -> bool {
        self.paths.remove(path)
    }

    /// Check if a path is tracked.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.paths.contains(path)
    }

    /// Check if there are any tracked paths.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Get all tracked paths.
    #[must_use]
    pub const fn paths(&self) -> &BTreeSet<PathBuf> {
        &self.paths
    }

    /// Walk all tracked paths on disk.
    #[must_use]
    pub fn scan(&self, work_tree: &Path) -> Scan {
        scan(work_tree, &self.paths.iter().collect::<Vec<_>>())
    }
}

/// What a walk of tracked paths found on disk.
#[derive(Default)]
pub struct Scan {
    /// Files (and symlinks) under the tracked paths.
    pub files: Vec<PathBuf>,
    /// Git repositories and worktrees nested inside tracked directories
    /// (e.g. worktrees under `.claude/worktrees/`). They are not ours to
    /// version: `git add` would record them as gitlinks.
    pub nested_repos: Vec<PathBuf>,
}

/// Walk `paths` (relative to `work_tree`), recursing into directories.
///
/// Symlinks are not followed, matching what `git add` records. Paths
/// missing from disk are skipped (they are handled as deletions).
#[must_use]
pub fn scan(work_tree: &Path, paths: &[&PathBuf]) -> Scan {
    let mut found = Scan::default();
    for path in paths {
        let Ok(meta) = fs::symlink_metadata(work_tree.join(path)) else {
            continue;
        };
        if meta.is_dir() {
            walk(&work_tree.join(path), path, &mut found);
        } else {
            found.files.push((*path).clone());
        }
    }
    found
}

fn walk(dir: &Path, relative_base: &Path, found: &mut Scan) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let relative = relative_base.join(entry.file_name());
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            found.files.push(relative);
        } else if entry.path().join(".git").exists() {
            found.nested_repos.push(relative);
        } else {
            walk(&entry.path(), &relative, found);
        }
    }
}
