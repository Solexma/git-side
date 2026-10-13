mod common;

use common::{TestEnv, write};

fn setup_remote(env: &TestEnv, p: &std::path::Path) -> String {
    let remote = env.root.join("remote.git");
    env.git(&env.root, &["init", "-q", "--bare", remote.to_str().unwrap()]);
    env.side(p, &["remote", "add", "origin", remote.to_str().unwrap()]);
    remote.to_string_lossy().into_owned()
}

#[test]
fn new_side_repo_uses_main_and_pushes() {
    // The harness sets init.defaultBranch=master on purpose.
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    assert_eq!(env.side_git(&["symbolic-ref", "HEAD"]), "refs/heads/main");

    let remote = setup_remote(&env, &p);
    env.side(&p, &["push"]);
    let pushed = env.git(&env.root, &["--git-dir", &remote, "rev-parse", "main"]);
    assert_eq!(pushed, env.side_git(&["rev-parse", "HEAD"]));
}

#[test]
fn legacy_master_branch_is_renamed_to_main() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "base"]);
    // Simulate a side repo created by an older version.
    env.side_git(&["branch", "-m", "main", "master"]);

    let remote = setup_remote(&env, &p);
    env.side(&p, &["push"]);

    assert_eq!(env.side_git(&["symbolic-ref", "HEAD"]), "refs/heads/main");
    env.git(&env.root, &["--git-dir", &remote, "rev-parse", "--verify", "main"]);
}
