use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{self, hash_path};
use crate::error::{Error, Result};
use crate::git;
use crate::tracked;

/// Represents a side repository for a project.
///
/// Linked worktrees of the project get their own `HEAD`, index and
/// `.side-tracked` inside the side repo, using git's own layout:
/// `<common_dir>/worktrees/<name>/`, on branch `wt/<name>`.
pub struct SideRepo {
    /// Git dir used for commands: `common_dir`, or the per-worktree admin dir.
    pub git_dir: PathBuf,
    /// Path to the bare git repository shared by all worktrees.
    pub common_dir: PathBuf,
    /// Path to the work tree (the current project worktree).
    pub work_tree: PathBuf,
    /// The initial commit SHA of the main repo (project identifier).
    pub root_sha: String,
    /// Name of the linked worktree, `None` in the main worktree.
    pub worktree: Option<String>,
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

        let common_dir = base_path.join(&root_sha);
        let linked = git::linked_worktree_name()?;
        let git_dir = linked
            .as_ref()
            .map_or_else(|| common_dir.clone(), |name| common_dir.join("worktrees").join(name));

        let repo = Self {
            git_dir,
            common_dir,
            work_tree,
            root_sha,
            worktree: linked,
        };
        if repo.has_side_repo() {
            repo.ensure_main_branch()?;
        }
        if repo.is_initialized() {
            repo.ensure_config()?;
            repo.write_worktree_gitdir()?;
        }
        Ok(repo)
    }

    /// Check if the side repo (and, in a linked worktree, its per-worktree
    /// state) has been initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.git_dir.join("HEAD").exists()
    }

    /// Check if the shared side repo exists, regardless of worktree state.
    #[must_use]
    pub fn has_side_repo(&self) -> bool {
        self.common_dir.join("HEAD").exists()
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

        if !self.has_side_repo() {
            // Create parent directory
            if let Some(parent) = self.common_dir.parent() {
                fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }

            // Initialize bare repo; push/pull operate on `main`, whatever the
            // user's init.defaultBranch says.
            let common_dir_str = self.common_dir.to_string_lossy();
            git::run(&["init", "--bare", "--initial-branch=main", &common_dir_str])?;
        }

        let Some(name) = &self.worktree else {
            return self.ensure_config();
        };
        // A half-initialized worktree (HEAD written, index not populated)
        // would make the next auto record every side file as deleted.
        self.init_worktree(name).inspect_err(|_| {
            let _ = fs::remove_dir_all(&self.git_dir);
        })
    }

    /// Set up per-worktree state for linked worktree `name`.
    ///
    /// The branch `wt/<name>` starts from `main`. A leftover branch from a
    /// previous worktree with the same name is reused only if it holds work
    /// not merged into `main`. Side files missing from the worktree are then
    /// checked out, so that "never checked out here" is not mistaken for
    /// "deleted".
    fn init_worktree(&self, name: &str) -> Result<()> {
        let branch = format!("refs/heads/wt/{name}");
        let has_main = self.ref_exists("refs/heads/main");
        if self.ref_exists(&branch) {
            let merged = has_main
                && self
                    .git_common(&["merge-base", "--is-ancestor", &branch, "refs/heads/main"])
                    .is_ok();
            if merged {
                self.git_common(&["update-ref", &branch, "refs/heads/main"])?;
            } else {
                eprintln!("warning: reusing unmerged side branch wt/{name}");
            }
        } else if has_main {
            self.git_common(&["update-ref", &branch, "refs/heads/main"])?;
        }

        write_file(&self.git_dir.join("commondir"), "../..\n")?;
        self.write_worktree_gitdir()?;
        write_file(&self.git_dir.join("HEAD"), &format!("ref: {branch}\n"))?;
        self.ensure_config()?;

        if self.git(&["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok() {
            self.git(&["read-tree", "HEAD"])?;
            if !self.git(&["ls-files", "--cached", "--", ".side-tracked"])?.is_empty() {
                self.git(&["update-index", "--skip-worktree", ".side-tracked"])?;
            }
            self.sync_tracked_file()?;
            self.materialize()?;
        }
        Ok(())
    }

    /// Point the side worktree's `gitdir` at the project worktree's `.git`
    /// file. While that file exists, `git worktree prune` (and `gc`) keep the
    /// per-worktree state; once the worktree is removed, git prunes it.
    fn write_worktree_gitdir(&self) -> Result<()> {
        if self.worktree.is_none() {
            return Ok(());
        }
        let gitdir_file = self.git_dir.join("gitdir");
        let content = format!("{}\n", self.work_tree.join(".git").display());
        if fs::read_to_string(&gitdir_file).is_ok_and(|c| c == content) {
            return Ok(());
        }
        write_file(&gitdir_file, &content)
    }

    /// Check out side files missing from the work-tree. Files that exist are
    /// never overwritten. Returns the paths written.
    ///
    /// # Errors
    ///
    /// Returns an error if the git commands fail.
    pub fn materialize(&self) -> Result<Vec<String>> {
        let missing = self.git(&["ls-files", "-z", "--deleted"])?;
        let files: Vec<String> = missing
            .split('\0')
            .filter(|f| !f.is_empty() && *f != ".side-tracked")
            .map(str::to_string)
            .collect();
        if !files.is_empty() {
            let mut args = vec!["checkout-index", "--"];
            args.extend(files.iter().map(String::as_str));
            self.git(&args)?;
        }
        // Record stat info so status does not rehash everything; exits
        // non-zero when files differ from the index, which is fine.
        let _ = self.git(&["update-index", "-q", "--refresh"]);
        Ok(files)
    }

    /// One-time configuration of the side repo (and of each worktree).
    ///
    /// Untracked files are hidden: from the side repo's point of view the
    /// whole project is untracked.
    ///
    /// `.side-tracked` is versioned in the side repo but lives in its git
    /// dir. A non-cone sparse checkout excluding it stops git from writing it
    /// into the project on `reset --hard`/`merge` and from reporting it as
    /// deleted. `merge=union` lets path lists from different branches merge
    /// cleanly.
    fn ensure_config(&self) -> Result<()> {
        // Per worktree: lives in the admin dir for linked worktrees.
        let sparse_file = self.git_dir.join("info").join("sparse-checkout");
        if sparse_file.exists() {
            return Ok(());
        }

        self.git(&["config", "status.showUntrackedFiles", "no"])?;
        self.git(&["config", "core.sparseCheckout", "true"])?;
        self.git(&["config", "core.sparseCheckoutCone", "false"])?;
        write_file(&sparse_file, "/*\n!/.side-tracked\n")?;

        let attributes = self.common_dir.join("info").join("attributes");
        let existing = fs::read_to_string(&attributes).unwrap_or_default();
        if !existing.contains(".side-tracked") {
            write_file(&attributes, &format!("{existing}.side-tracked merge=union\n"))?;
        }

        if self.git(&["ls-files", "--cached", "--", ".side-tracked"])?.is_empty() {
            return Ok(());
        }
        self.git(&["update-index", "--skip-worktree", ".side-tracked"])?;
        Ok(())
    }

    /// Side repos created by older versions follow `init.defaultBranch`.
    /// Rename their branch to `main` unless a `main` branch already exists.
    fn ensure_main_branch(&self) -> Result<()> {
        let head = fs::read_to_string(self.common_dir.join("HEAD")).unwrap_or_default();
        let Some(branch) = head.trim().strip_prefix("ref: refs/heads/") else {
            return Ok(());
        };
        if branch == "main" || self.ref_exists("refs/heads/main") {
            return Ok(());
        }

        if self.ref_exists(&format!("refs/heads/{branch}")) {
            self.git_common(&["branch", "-m", branch, "main"])?;
        } else {
            // Unborn branch: just repoint HEAD.
            self.git_common(&["symbolic-ref", "HEAD", "refs/heads/main"])?;
        }
        Ok(())
    }

    /// Check whether a ref exists in the shared side repo.
    #[must_use]
    pub fn ref_exists(&self, refname: &str) -> bool {
        self.git_common(&["rev-parse", "--verify", "--quiet", refname]).is_ok()
    }

    /// Check whether a side merge is waiting for conflict resolution.
    #[must_use]
    pub fn merge_in_progress(&self) -> bool {
        self.git_dir.join("MERGE_HEAD").exists()
    }

    /// Fail unless running in the main worktree.
    ///
    /// # Errors
    ///
    /// Returns `MainWorktreeOnly` in a linked worktree.
    pub const fn require_main_worktree(&self, command: &'static str) -> Result<()> {
        if self.worktree.is_some() {
            return Err(Error::MainWorktreeOnly(command));
        }
        Ok(())
    }

    /// Worktree side branches (`wt/*`) with commits not merged into `main`.
    #[must_use]
    pub fn unmerged_worktree_branches(&self) -> Vec<String> {
        self.git_common(&[
            "for-each-ref",
            "--format=%(refname:short)",
            "--no-merged=refs/heads/main",
            "refs/heads/wt/",
        ])
        .map(|out| out.lines().map(str::to_string).collect())
        .unwrap_or_default()
    }

    /// Run a git command in the context of the side repo.
    ///
    /// # Errors
    ///
    /// Returns an error if the git command fails.
    pub fn git(&self, args: &[&str]) -> Result<String> {
        git::run_with_paths(&self.git_dir, &self.work_tree, args)
    }

    /// Run a git command against the shared side repo, bypassing
    /// per-worktree state.
    fn git_common(&self, args: &[&str]) -> Result<String> {
        git::run_with_paths(&self.common_dir, &self.work_tree, args)
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
        let path = path.to_path_buf();
        self.add(&["-f"], &[&path])
    }

    /// Stage paths with update flag (handles modifications and deletions).
    ///
    /// Only paths with entries in the index are passed: a pathspec that
    /// matches nothing makes git abort the whole command.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization or staging fails.
    pub fn stage_update(&self, paths: &[PathBuf]) -> Result<()> {
        self.ensure_initialized()?;

        let indexed = self.git(&["ls-files", "--cached"])?;
        let paths: Vec<&PathBuf> = paths
            .iter()
            .filter(|p| indexed.lines().any(|entry| Path::new(entry).starts_with(p)))
            .collect();
        if paths.is_empty() {
            return Ok(());
        }

        // Drop gitlinks recorded for nested repos by older versions.
        let nested = tracked::nested_repos(&self.work_tree, &paths);
        if !nested.is_empty() {
            let nested_strs: Vec<String> = nested.iter().map(|p| p.to_string_lossy().into_owned()).collect();
            let mut args = vec!["rm", "--cached", "-r", "-q", "--ignore-unmatch", "--"];
            args.extend(nested_strs.iter().map(String::as_str));
            self.git(&args)?;
        }

        self.add(&["-f", "-u"], &paths)
    }

    /// Stage paths (adds new files). Paths missing from disk are skipped;
    /// their deletion is handled by `stage_update`.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization or staging fails.
    pub fn stage_new(&self, paths: &[PathBuf]) -> Result<()> {
        let paths: Vec<&PathBuf> = paths
            .iter()
            .filter(|p| self.work_tree.join(p).exists())
            .collect();
        if paths.is_empty() {
            return Ok(());
        }
        self.ensure_initialized()?;
        self.add(&["-f"], &paths)
    }

    /// Run `git add <flags> -- <paths>`, excluding repositories nested
    /// inside the paths so they are never recorded as gitlinks.
    fn add(&self, flags: &[&str], paths: &[&PathBuf]) -> Result<()> {
        let mut pathspecs: Vec<String> = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        pathspecs.extend(
            tracked::nested_repos(&self.work_tree, paths)
                .iter()
                .map(|p| format!(":(exclude){}", p.to_string_lossy())),
        );

        let mut args = vec!["add"];
        args.extend_from_slice(flags);
        args.push("--");
        args.extend(pathspecs.iter().map(String::as_str));

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

        // Check if there's anything staged to commit
        // diff --cached --quiet exits with 1 if there are staged changes, 0 if none
        let has_staged = self.git(&["diff", "--cached", "--quiet"]).is_err();
        if !has_staged {
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
            if self.worktree.is_some() && self.has_side_repo() {
                return Ok(String::from(
                    "Side files are not set up in this worktree yet. Use 'git side checkout' to check them out.",
                ));
            }
            return Ok(String::from("Side repo not initialized. Use 'git side add <path>' to start tracking files."));
        }
        let output = self.git(&["status"])?;
        // Implementation detail of how .side-tracked is kept out of the work-tree.
        Ok(output
            .lines()
            .filter(|line| !line.starts_with("You are in a sparse checkout"))
            .collect::<Vec<_>>()
            .join("\n"))
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

    /// Force push to origin/main.
    ///
    /// # Errors
    ///
    /// Returns an error if the push fails (e.g. no remote configured).
    pub fn push(&self) -> Result<()> {
        self.git(&["push", "-u", "--force", "origin", "main"])?;
        Ok(())
    }

    /// List files in the side repo's index. Mirrors `git ls-files`.
    ///
    /// Returns an empty string if the side repo is not initialized,
    /// so scripting consumers can pipe safely on fresh clones.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `git ls-files` command fails.
    pub fn ls_files(&self, args: &[&str]) -> Result<String> {
        if !self.is_initialized() {
            return Ok(String::new());
        }
        let mut ls_args = vec!["ls-files"];
        ls_args.extend(args);
        self.git(&ls_args)
    }

    /// Stage the .side-tracked file using git plumbing.
    /// Since `.side-tracked` lives in `git_dir` (not `work_tree`), we use `hash-object` + `update-index`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist or git commands fail.
    pub fn stage_tracked_file(&self) -> Result<()> {
        let tracked_file = self.tracked_file();
        if !tracked_file.exists() {
            return Ok(());
        }

        // Hash the file and write to object store
        let tracked_path_str = tracked_file.to_string_lossy();
        let sha = self.git(&["hash-object", "-w", &tracked_path_str])?;
        let sha = sha.trim();

        // Add to index with name .side-tracked at repo root, never checked out
        let cacheinfo = format!("100644,{sha},.side-tracked");
        self.git(&["update-index", "--add", "--cacheinfo", &cacheinfo])?;
        self.git(&["update-index", "--skip-worktree", ".side-tracked"])?;

        Ok(())
    }

    /// Write the index version of `.side-tracked` back to the git dir.
    /// Needed after anything that rewrites the index from a tree (pull, merge).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn sync_tracked_file(&self) -> Result<()> {
        // Not in the index: nothing tracked yet on this branch.
        self.git(&["show", ":.side-tracked"])
            .map_or(Ok(()), |content| write_file(&self.tracked_file(), &content))
    }
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    fs::write(path, content).map_err(|e| Error::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}
