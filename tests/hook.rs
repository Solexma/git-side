mod common;

use common::TestEnv;

#[test]
fn hook_installed_from_linked_worktree_lands_in_common_hooks_dir() {
    let env = TestEnv::new();
    let p = env.project("p");
    let wt = env.root.join("wt");
    env.git(&p, &["worktree", "add", "-q", wt.to_str().unwrap()]);

    env.side(&wt, &["hook", "install"]);

    assert!(p.join(".git/hooks/post-commit").exists());
    assert!(!p.join(".git/worktrees/wt/hooks/post-commit").exists());

    env.side(&wt, &["hook", "uninstall"]);
    assert!(!p.join(".git/hooks/post-commit").exists());
}

#[test]
fn hook_respects_core_hooks_path() {
    let env = TestEnv::new();
    let p = env.project("p");
    env.git(&p, &["config", "core.hooksPath", ".githooks"]);
    let sub = p.join("sub");
    std::fs::create_dir_all(&sub).unwrap();

    env.side(&sub, &["hook", "install"]);

    assert!(p.join(".githooks/post-commit").exists());
}
