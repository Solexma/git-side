use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::side_repo::SideRepo;

/// Manages the .side-tracked file.
pub struct TrackedPaths {
    file_path: PathBuf,
    paths: BTreeSet<PathBuf>,
}

impl TrackedPaths {
    /// Load tracked paths from the side repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the tracked file exists but cannot be read.
    pub fn load(repo: &SideRepo) -> Result<Self> {
        let file_path = repo.tracked_file();
        let paths = if file_path.exists() {
            let content = fs::read_to_string(&file_path).map_err(|e| Error::ReadFile {
                path: file_path.clone(),
                source: e,
            })?;
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(PathBuf::from)
                .collect()
        } else {
            BTreeSet::new()
        };

        Ok(Self { file_path, paths })
    }

    /// Save tracked paths to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content: String = self
            .paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&self.file_path, content).map_err(|e| Error::WriteFile {
            path: self.file_path.clone(),
            source: e,
        })
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

    /// Expand all tracked paths to actual files on disk.
    /// Directories are walked recursively.
    #[must_use]
    pub fn expand(&self, work_tree: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for path in &self.paths {
            let full_path = work_tree.join(path);
            if full_path.is_file() {
                files.push(path.clone());
            } else if full_path.is_dir() {
                Self::walk_dir(&full_path, path, &mut files);
            }
            // If path doesn't exist, skip it (will be handled as deletion)
        }

        files
    }

    /// Recursively walk a directory and collect all files.
    fn walk_dir(dir: &Path, relative_base: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let relative = relative_base.join(entry.file_name());

                if entry_path.is_file() {
                    files.push(relative);
                } else if entry_path.is_dir() {
                    Self::walk_dir(&entry_path, &relative, files);
                }
            }
        }
    }
}
