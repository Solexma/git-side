//! Shared helpers for integration tests.
//!
//! Every `TestEnv` gets its own temp directory used as `HOME` and XDG base,
//! so the real user config and side repos are never touched.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct TestEnv {
    pub root: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("git-side-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("home")).unwrap();
        fs::write(
            root.join("home/.gitconfig"),
            "[user]\n\tname = Test\n\temail = test@example.com\n\
             [init]\n\tdefaultBranch = master\n\
             [advice]\n\tdefaultBranchName = false\n",
        )
        .unwrap();
        Self { root }
    }

    fn apply_env(&self, cmd: &mut Command) {
        let home = self.root.join("home");
        cmd.env("HOME", &home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("XDG_DATA_HOME", home.join(".local/share"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE");
    }

    /// Run `git` in `dir`, panicking on failure.
    pub fn git(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.git_raw(dir, args);
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    pub fn git_raw(&self, dir: &Path, args: &[&str]) -> Output {
        let mut cmd = Command::new("git");
        cmd.current_dir(dir).args(args);
        self.apply_env(&mut cmd);
        cmd.output().unwrap()
    }

    /// Run `git-side` in `dir`, returning the raw output.
    pub fn side_raw(&self, dir: &Path, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-side"));
        cmd.current_dir(dir).args(args);
        self.apply_env(&mut cmd);
        cmd.output().unwrap()
    }

    /// Run `git-side` in `dir` with extra environment variables.
    pub fn side_env(&self, dir: &Path, args: &[&str], vars: &[(&str, &str)]) -> String {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-side"));
        cmd.current_dir(dir).args(args);
        self.apply_env(&mut cmd);
        cmd.envs(vars.iter().copied());
        let out = cmd.output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Run `git-side` in `dir`, panicking on failure; returns stdout.
    pub fn side(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.side_raw(dir, args);
        assert!(
            out.status.success(),
            "git side {args:?} failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Create a project repo with one empty commit.
    pub fn project(&self, name: &str) -> PathBuf {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir).unwrap();
        self.git(&dir, &["init", "-q"]);
        self.git(&dir, &["commit", "-q", "--allow-empty", "-m", "init"]);
        dir
    }

    /// Path of the (single) side repo created in this environment.
    pub fn side_dir(&self) -> PathBuf {
        let base = self.root.join("home/.local/share/git-side");
        let mut entries: Vec<_> = fs::read_dir(&base).unwrap().flatten().collect();
        assert_eq!(entries.len(), 1, "expected exactly one side repo");
        entries.pop().unwrap().path()
    }

    /// Run git directly against the side repo's common dir.
    pub fn side_git(&self, args: &[&str]) -> String {
        let side = self.side_dir();
        let mut full = vec!["--git-dir", side.to_str().unwrap()];
        full.extend_from_slice(args);
        self.git(&self.root, &full)
    }

    pub fn ls_files(&self, dir: &Path) -> Vec<String> {
        self.side(dir, &["ls-files"])
            .lines()
            .map(str::to_string)
            .collect()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}
