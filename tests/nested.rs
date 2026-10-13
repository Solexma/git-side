mod common;

use common::{TestEnv, write};

#[test]
fn nested_worktree_inside_tracked_dir_is_not_committed() {
    // Claude Code creates worktrees under .claude/worktrees/.
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join(".claude/settings.json"), "{}");
    env.side(&p, &["add", ".claude"]);
    env.side(&p, &["commit", "-m", "base"]);

    env.git(&p, &["worktree", "add", "-q", ".claude/worktrees/agent1"]);
    write(&p.join(".claude/worktrees/agent1/scratch.txt"), "x");
    write(&p.join(".claude/agents.md"), "new");
    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "touch"]);
    let out = env.side(&p, &["auto"]);

    let index = env.side_git(&["ls-tree", "-r", "HEAD"]);
    assert!(!index.contains("160000"), "gitlink committed:\n{index}");
    assert!(!index.contains("worktrees"), "{index}");
    assert!(index.contains(".claude/agents.md"), "{index}");
    assert!(out.contains("2 file(s)"), "{out}");
}

#[test]
fn existing_gitlink_is_dropped_on_auto() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join(".claude/settings.json"), "{}");
    env.side(&p, &["add", ".claude"]);
    env.side(&p, &["commit", "-m", "base"]);

    // State left behind by older versions.
    env.git(&p, &["worktree", "add", "-q", ".claude/worktrees/agent1"]);
    let head = env.git(&p, &["rev-parse", "HEAD"]);
    let side = env.side_dir();
    env.git(
        &p,
        &[
            "--git-dir",
            side.to_str().unwrap(),
            "--work-tree",
            p.to_str().unwrap(),
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{head},.claude/worktrees/agent1"),
        ],
    );
    env.side(&p, &["commit", "-m", "gitlink"]);
    assert!(env.side_git(&["ls-tree", "-r", "HEAD"]).contains("160000"));

    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "touch"]);
    env.side(&p, &["auto"]);

    assert!(!env.side_git(&["ls-tree", "-r", "HEAD"]).contains("160000"));
}
