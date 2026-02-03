use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{self, hash_path};
use crate::error::{Error, Result};
use crate::git;

/// Represents a side repository for a project.
pub struct SideRepo {
    /// Path to the bare git repository.
    pub git_dir: PathBuf,
    /// Path to the work tree (the main project directory).
    pub work_tree: PathBuf,
    /// The initial commit SHA of the main repo (project identifier).
    pub root_sha: String,
}

impl SideRepo {
    /// Resolve or create a side repo for the current project.
    ///
    /// # Errors
    ///
    /// Returns an error if not in a git repository or if config files cannot be accessed.
    pub fn open() -> Result<Self> {
        let work_tree = git::repo_root()?;
        let path_hash = hash_path(&work_tree);

        // Try cache first
        let root_sha = if let Some(sha) = config::cache_lookup(&path_hash)? {
            sha
        } else {
            // Cache miss: resolve and store
            let sha = git::initial_commit_sha()?;
            config::cache_store(&path_hash, &sha)?;
            sha
        };

        // Get base path (custom or default)
        let base_path = config::paths_lookup(&root_sha)?
            .unwrap_or_else(config::default_base_path);

        let git_dir = base_path.join(&root_sha);

        Ok(Self {
            git_dir,
            work_tree,
            root_sha,
        })
    }

    /// Check if the side repo has been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.git_dir.exists() && self.git_dir.join("HEAD").exists()
    }

    /// Initialize the side repo if not already done.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or git init fails.
    pub fn ensure_initialized(&self) -> Result<()> {
        if self.is_initialized() {
            return Ok(());
        }

        // Create parent directory
        if let Some(parent) = self.git_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Initialize bare repo
        let git_dir_str = self.git_dir.to_string_lossy();
        git::run(&["init", "--bare", &git_dir_str])?;

        Ok(())
    }

    /// Run a git command in the context of the side repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the git command fails.
    pub fn git(&self, args: &[&str]) -> Result<String> {
        git::run_with_paths(&self.git_dir, &self.work_tree, args)
    }

    /// Get the path to the .side-tracked file.
    #[must_use]
    pub fn tracked_file(&self) -> PathBuf {
        self.git_dir.join(".side-tracked")
    }

    /// Stage a path (forced, bypassing gitignore).
    ///
    /// # Errors
    ///
    /// Returns an error if initialization or staging fails.
    pub fn stage(&self, path: &Path) -> Result<()> {
        self.ensure_initialized()?;
        let path_str = path.to_string_lossy();
        self.git(&["add", "-f", &path_str])?;
        Ok(())
    }

    /// Stage paths with update flag (handles modifications and deletions).
    ///
    /// # Errors
    ///
    /// Returns an error if initialization or staging fails.
    pub fn stage_update(&self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }
        self.ensure_initialized()?;

        let path_strs: Vec<String> = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let mut args: Vec<&str> = vec!["add", "-f", "-u", "--"];
        args.extend(path_strs.iter().map(String::as_str));

        self.git(&args)?;
        Ok(())
    }

    /// Stage paths (adds new files).
    ///
    /// # Errors
    ///
    /// Returns an error if initialization or staging fails.
    pub fn stage_new(&self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }
        self.ensure_initialized()?;

        let path_strs: Vec<String> = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let mut args: Vec<&str> = vec!["add", "-f", "--"];
        args.extend(path_strs.iter().map(String::as_str));

        self.git(&args)?;
        Ok(())
    }

    /// Commit staged changes.
    ///
    /// # Errors
    ///
    /// Returns `NothingToCommit` if there are no staged changes, or an error if commit fails.
    pub fn commit(&self, message: &str) -> Result<()> {
        self.ensure_initialized()?;

        // Check if there's anything to commit
        let status = self.git(&["status", "--porcelain"])?;
        if status.is_empty() {
            return Err(Error::NothingToCommit);
        }

        self.git(&["commit", "-m", message])?;
        Ok(())
    }

    /// Get status output.
    ///
    /// # Errors
    ///
    /// Returns an error if the git status command fails.
    pub fn status(&self) -> Result<String> {
        if !self.is_initialized() {
            return Ok(String::from("Side repo not initialized. Use 'git side add <path>' to start tracking files."));
        }
        self.git(&["status"])
    }

    /// Get log output.
    ///
    /// # Errors
    ///
    /// Returns an error if the git log command fails.
    pub fn log(&self, args: &[&str]) -> Result<String> {
        if !self.is_initialized() {
            return Ok(String::from("Side repo not initialized. No history yet."));
        }

        let mut log_args = vec!["log"];
        log_args.extend(args);
        self.git(&log_args)
    }

    /// Remove a path from the index (unstage).
    ///
    /// # Errors
    ///
    /// Returns an error if the git rm command fails (though failures are typically ignored).
    pub fn unstage(&self, path: &Path) -> Result<()> {
        if !self.is_initialized() {
            return Ok(());
        }
        let path_str = path.to_string_lossy();
        // Use rm --cached to remove from index without deleting the file
        let _ = self.git(&["rm", "--cached", "-r", "--ignore-unmatch", &path_str]);
        Ok(())
    }
}
