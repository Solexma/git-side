mod common;

use std::fs;
use std::path::PathBuf;

use common::{TestEnv, write};

/// Project with CLAUDE.md and .claude/ committed to the side repo,
/// plus a linked worktree named `wt`.
fn setup(env: &TestEnv) -> (PathBuf, PathBuf) {
    let p = env.project("p");
    write(&p.join("CLAUDE.md"), "rules v1");
    write(&p.join(".claude/settings.json"), "{}");
    env.side(&p, &["add", "CLAUDE.md"]);
    env.side(&p, &["add", ".claude"]);
    env.side(&p, &["commit", "-m", "base"]);

    let wt = env.root.join("wt");
    env.git(&p, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    (p, wt)
}

fn main_tree(env: &TestEnv) -> String {
    env.side_git(&["ls-tree", "-r", "--name-only", "main"])
}

#[test]
fn checkout_materialises_side_files_in_worktree() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);

    env.side(&wt, &["checkout"]);

    assert_eq!(fs::read_to_string(wt.join("CLAUDE.md")).unwrap(), "rules v1");
    assert!(wt.join(".claude/settings.json").exists());
    assert!(!wt.join(".side-tracked").exists());
    let status = env.side(&wt, &["status"]);
    assert!(status.contains("On branch wt/wt"), "{status}");
    assert!(status.contains("nothing to commit"), "{status}");
    // The main worktree is untouched.
    assert!(env.side(&p, &["status"]).contains("nothing to commit"));
}

#[test]
fn add_works_in_worktree_without_touching_main() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);

    write(&wt.join("CLAUDE.md"), "rules from wt");
    env.side(&wt, &["add", "CLAUDE.md"]);
    write(&wt.join("docs/spec.md"), "spec");
    env.side(&wt, &["add", "docs"]);
    env.side(&wt, &["commit", "-m", "wt work"]);

    let wt_tree = env.side_git(&["ls-tree", "-r", "--name-only", "wt/wt"]);
    assert!(wt_tree.contains("docs/spec.md"), "{wt_tree}");
    assert!(wt_tree.contains(".claude/settings.json"), "{wt_tree}");
    assert!(!main_tree(&env).contains("docs/spec.md"));
    assert_eq!(env.side_git(&["show", "main:CLAUDE.md"]), "rules v1");
    // Main's tracked list is its own.
    let info = env.side(&p, &["info"]);
    assert!(!info.contains("- docs"), "{info}");
}

#[test]
fn auto_in_fresh_worktree_does_not_record_deletions() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);

    // What the shared post-commit hook does on the first commit in the worktree.
    env.git(&wt, &["commit", "-q", "--allow-empty", "-m", "wt commit"]);
    env.side(&wt, &["auto"]);

    let wt_tree = env.side_git(&["ls-tree", "-r", "--name-only", "wt/wt"]);
    assert!(wt_tree.contains("CLAUDE.md"), "{wt_tree}");
    assert!(wt_tree.contains(".claude/settings.json"), "{wt_tree}");
    assert!(main_tree(&env).contains("CLAUDE.md"));
    assert!(env.side(&p, &["status"]).contains("nothing to commit"));
}

#[test]
fn existing_files_in_worktree_are_not_overwritten() {
    let env = TestEnv::new();
    let (_p, wt) = setup(&env);
    write(&wt.join("CLAUDE.md"), "local edit");

    env.side(&wt, &["checkout"]);

    assert_eq!(fs::read_to_string(wt.join("CLAUDE.md")).unwrap(), "local edit");
    assert!(env.side(&wt, &["status"]).contains("modified:   CLAUDE.md"));
}

#[test]
fn merge_worktree_branch_into_main() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("CLAUDE.md"), "rules v2");
    write(&wt.join("docs/spec.md"), "spec");
    env.side(&wt, &["add", "CLAUDE.md"]);
    env.side(&wt, &["add", "docs"]);
    env.side(&wt, &["commit", "-m", "wt work"]);

    env.side(&p, &["merge", "wt"]);

    assert_eq!(fs::read_to_string(p.join("CLAUDE.md")).unwrap(), "rules v2");
    assert_eq!(fs::read_to_string(p.join("docs/spec.md")).unwrap(), "spec");
    assert!(!p.join(".side-tracked").exists());
    let info = env.side(&p, &["info"]);
    assert!(info.contains("- docs"), "tracked list not merged:\n{info}");
    assert!(env.side(&p, &["status"]).contains("nothing to commit"));
}

#[test]
fn merge_main_into_worktree() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    env.side(&wt, &["checkout"]);

    write(&p.join("NOTES.md"), "notes");
    env.side(&p, &["add", "NOTES.md"]);
    env.side(&p, &["commit", "-m", "notes"]);

    env.side(&wt, &["merge", "main"]);

    assert_eq!(fs::read_to_string(wt.join("NOTES.md")).unwrap(), "notes");
    assert!(env.side(&wt, &["info"]).contains("- NOTES.md"));
}

#[test]
fn merge_conflict_then_continue() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("CLAUDE.md"), "from wt");
    env.side(&wt, &["add", "CLAUDE.md"]);
    env.side(&wt, &["commit", "-m", "wt"]);
    write(&p.join("CLAUDE.md"), "from main");
    env.side(&p, &["add", "CLAUDE.md"]);
    env.side(&p, &["commit", "-m", "main"]);

    let out = env.side_raw(&p, &["merge", "wt"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("CLAUDE.md") && stderr.contains("--continue"), "{stderr}");

    write(&p.join("CLAUDE.md"), "resolved");
    env.side(&p, &["merge", "--continue"]);

    assert_eq!(env.side_git(&["show", "main:CLAUDE.md"]), "resolved");
    let parents = env.side_git(&["rev-list", "--parents", "-n1", "main"]);
    assert_eq!(parents.split_whitespace().count(), 3, "not a merge commit");
}

#[test]
fn merge_abort_restores_state() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("CLAUDE.md"), "from wt");
    env.side(&wt, &["add", "CLAUDE.md"]);
    env.side(&wt, &["commit", "-m", "wt"]);
    write(&p.join("CLAUDE.md"), "from main");
    env.side(&p, &["add", "CLAUDE.md"]);
    env.side(&p, &["commit", "-m", "main"]);

    assert!(!env.side_raw(&p, &["merge", "wt"]).status.success());
    env.side(&p, &["merge", "--abort"]);

    assert_eq!(fs::read_to_string(p.join("CLAUDE.md")).unwrap(), "from main");
    assert!(env.side(&p, &["status"]).contains("nothing to commit"));
}

#[test]
fn merge_rejects_unknown_and_self() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    env.side(&wt, &["checkout"]);

    assert!(!env.side_raw(&p, &["merge", "nope"]).status.success());
    assert!(!env.side_raw(&p, &["merge", "main"]).status.success());
    assert!(!env.side_raw(&wt, &["merge", "wt"]).status.success());
}

#[test]
fn push_and_pull_are_refused_in_worktree() {
    let env = TestEnv::new();
    let (_p, wt) = setup(&env);

    for cmd in ["push", "pull"] {
        let out = env.side_raw(&wt, &[cmd]);
        assert!(!out.status.success(), "{cmd} succeeded in a linked worktree");
        assert!(String::from_utf8_lossy(&out.stderr).contains("main worktree"));
    }
}

#[test]
fn removed_worktree_is_pruned_but_branch_kept() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("idea.md"), "idea");
    env.side(&wt, &["add", "idea.md"]);
    env.side(&wt, &["commit", "-m", "idea"]);

    // Still present while the worktree exists.
    env.side_git(&["worktree", "prune", "--expire=now"]);
    assert!(env.side_dir().join("worktrees/wt").exists());

    env.git(&p, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
    env.side_git(&["worktree", "prune", "--expire=now"]);
    assert!(!env.side_dir().join("worktrees/wt").exists());

    env.side_git(&["rev-parse", "--verify", "wt/wt"]);
    let info = env.side(&p, &["info"]);
    assert!(info.contains("wt/wt"), "unmerged branch not listed:\n{info}");
}

#[test]
fn recreated_worktree_starts_from_main_when_old_branch_was_merged() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("idea.md"), "idea");
    env.side(&wt, &["add", "idea.md"]);
    env.side(&wt, &["commit", "-m", "idea"]);
    env.side(&p, &["merge", "wt"]);
    write(&p.join("later.md"), "later");
    env.side(&p, &["add", "later.md"]);
    env.side(&p, &["commit", "-m", "later"]);

    env.git(&p, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
    env.side_git(&["worktree", "prune", "--expire=now"]);
    env.git(&p, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    env.side(&wt, &["checkout"]);

    assert_eq!(fs::read_to_string(wt.join("later.md")).unwrap(), "later");
}

#[test]
fn auto_does_not_commit_conflict_markers_during_side_merge() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    write(&wt.join("CLAUDE.md"), "from wt");
    env.side(&wt, &["add", "CLAUDE.md"]);
    env.side(&wt, &["commit", "-m", "wt"]);
    write(&p.join("CLAUDE.md"), "from main");
    env.side(&p, &["add", "CLAUDE.md"]);
    env.side(&p, &["commit", "-m", "main"]);
    assert!(!env.side_raw(&p, &["merge", "wt"]).status.success());
    let before = env.side_git(&["rev-parse", "main"]);

    // A main-repo commit fires the post-commit hook mid-merge.
    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "unrelated"]);
    env.side(&p, &["auto"]);

    assert_eq!(env.side_git(&["rev-parse", "main"]), before);
}

#[test]
fn merge_continue_without_merge_fails() {
    let env = TestEnv::new();
    let (p, _wt) = setup(&env);
    assert!(!env.side_raw(&p, &["merge", "--continue"]).status.success());
}

#[cfg(unix)]
#[test]
fn failed_worktree_setup_leaves_no_half_initialized_state() {
    use std::os::unix::fs::PermissionsExt;

    let env = TestEnv::new();
    let (_p, wt) = setup(&env);
    // A read-only worktree makes checking out the side files fail.
    let set_mode = |mode| fs::set_permissions(&wt, fs::Permissions::from_mode(mode)).unwrap();
    set_mode(0o555);

    let out = env.side_raw(&wt, &["checkout"]);
    set_mode(0o755);
    assert!(!out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
    assert!(!env.side_dir().join("worktrees/wt").exists());

    env.side(&wt, &["checkout"]);
    assert!(wt.join("CLAUDE.md").is_file());
}

#[test]
fn recreated_worktree_without_prune_does_not_delete_side_files() {
    let env = TestEnv::new();
    let (p, wt) = setup(&env);
    env.side(&wt, &["checkout"]);

    // No prune in between: the side state of the old worktree is still there.
    env.git(&p, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
    env.git(&p, &["worktree", "add", "-q", "-f", wt.to_str().unwrap()]);
    env.git(&wt, &["commit", "-q", "--allow-empty", "-m", "new wt"]);
    env.side(&wt, &["auto"]);

    let wt_tree = env.side_git(&["ls-tree", "-r", "--name-only", "wt/wt"]);
    assert!(wt_tree.contains("CLAUDE.md"), "{wt_tree}");
    assert!(wt.join("CLAUDE.md").exists());
}

#[test]
fn worktree_named_main_and_branch_names_from_info() {
    let env = TestEnv::new();
    let (p, _wt) = setup(&env);
    let main_wt = env.root.join("main");
    env.git(&p, &["worktree", "add", "-q", main_wt.to_str().unwrap()]);
    write(&main_wt.join("idea.md"), "idea");
    env.side(&main_wt, &["add", "idea.md"]);
    env.side(&main_wt, &["commit", "-m", "idea"]);

    // Inside worktree "main", merging the side branch `main` is not a self-merge.
    env.side(&main_wt, &["merge", "main"]);

    // The name printed by info works as-is.
    let info = env.side(&p, &["info"]);
    assert!(info.contains("wt/main"), "{info}");
    env.side(&p, &["merge", "wt/main"]);
    assert!(p.join("idea.md").exists());
}

#[test]
fn merge_without_side_repo_creates_nothing() {
    let env = TestEnv::new();
    let p = env.project("p");
    assert!(!env.side_raw(&p, &["merge", "x"]).status.success());
    assert!(!env.root.join("home/.local/share/git-side").exists());
}
