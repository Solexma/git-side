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

    #[error("path already tracked: {}", .0.display())]
    PathAlreadyTracked(PathBuf),

    #[error("path not tracked: {}", .0.display())]
    PathNotTracked(PathBuf),

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
