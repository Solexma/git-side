use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not in a git repository")]
    NotInGitRepo,

    #[error("no commits in repository (cannot determine project identity)")]
    NoCommits,

    #[error("git command failed: {0}")]
    GitCommandFailed(String),

    #[error("path not found: {}", .0.display())]
    PathNotFound(PathBuf),

    #[error("path is outside the repository: {}", .0.display())]
    PathOutsideRepo(PathBuf),

    #[error("refusing to track the whole repository")]
    PathIsRepoRoot,

    #[error("path not tracked: {}", .0.display())]
    PathNotTracked(PathBuf),

    #[error("'git side {0}' only works in the main worktree")]
    MainWorktreeOnly(&'static str),

    #[error("no side repo for this project yet (start with 'git side add <path>')")]
    NoSideRepo,

    #[error("no side branch '{0}' (expected 'main', 'wt/<name>' or a worktree name)")]
    UnknownSideBranch(String),

    #[error("cannot merge '{0}' into itself")]
    MergeIntoSelf(String),

    #[error("merge conflict in: {0}\nresolve the files, then run 'git side merge --continue' (or 'git side merge --abort')")]
    MergeConflict(String),

    #[error("no side merge in progress")]
    NoMergeInProgress,

    #[error("nothing to commit")]
    NothingToCommit,

    #[error("no tracked paths configured")]
    NoTrackedPaths,

    #[error("hook already installed: {0}")]
    HookAlreadyInstalled(String),

    #[error("hook not installed: {0}")]
    HookNotInstalled(String),

    #[error("failed to read {}: {source}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {}: {source}", path.display())]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create directory {}: {source}", path.display())]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config format in {}", path.display())]
    InvalidConfig { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;
