mod common;

use common::{TestEnv, write};

#[test]
fn status_does_not_report_side_tracked_as_deleted() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    let status = env.side(&p, &["status"]);
    assert!(!status.contains(".side-tracked"), "{status}");
    assert!(!status.contains("sparse checkout"), "{status}");
    assert!(status.contains("nothing to commit"), "{status}");
}

#[test]
fn status_hides_main_project_files() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("src/main.rs"), "fn main() {}");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    let status = env.side(&p, &["status"]);
    assert!(!status.contains("src/"), "{status}");
    assert!(!status.contains("Untracked"), "{status}");
}

#[test]
fn pull_restores_tracked_list_without_touching_work_tree() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);
    let remote = env.root.join("remote.git");
    env.git(&env.root, &["init", "-q", "--bare", remote.to_str().unwrap()]);
    env.side(&p, &["remote", "add", "origin", remote.to_str().unwrap()]);
    env.side(&p, &["push"]);

    // Local-only change that the pull must discard (remote wins).
    write(&p.join("b.txt"), "b");
    env.side(&p, &["add", "b.txt"]);
    env.side(&p, &["commit", "-m", "local"]);

    env.side(&p, &["pull"]);

    assert!(!p.join(".side-tracked").exists(), "pull wrote .side-tracked into the project");
    let info = env.side(&p, &["info"]);
    assert!(info.contains("- a.txt"), "{info}");
    assert!(!info.contains("- b.txt"), "local tracked list survived the pull:\n{info}");
}

#[test]
fn legacy_side_repo_is_upgraded_on_open() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    // Strip everything the upgrade is supposed to add back.
    let side = env.side_dir();
    std::fs::remove_file(side.join("info/sparse-checkout")).unwrap();
    env.side_git(&["config", "--unset", "core.sparseCheckout"]);
    env.git(
        &p,
        &[
            "--git-dir",
            side.to_str().unwrap(),
            "--work-tree",
            p.to_str().unwrap(),
            "update-index",
            "--no-skip-worktree",
            ".side-tracked",
        ],
    );

    let status = env.side(&p, &["status"]);
    assert!(!status.contains(".side-tracked"), "{status}");
}

#[test]
fn status_is_clean_with_localized_git() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    let status = env.side_env(&p, &["status"], &[("LC_ALL", "it_IT.UTF-8"), ("LANG", "it_IT.UTF-8")]);
    assert!(!status.to_lowercase().contains("sparse"), "{status}");
    assert!(!status.contains("\n\n\n"), "{status}");
}
