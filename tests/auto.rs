mod common;

use std::fs;

use common::{TestEnv, write};

#[test]
fn auto_records_deletion_of_tracked_file() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    write(&p.join("b.txt"), "b");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["add", "b.txt"]);
    env.side(&p, &["commit", "-m", "base"]);

    fs::remove_file(p.join("a.txt")).unwrap();
    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "drop a"]);
    env.side(&p, &["auto"]);

    let tree = env.side_git(&["ls-tree", "--name-only", "HEAD"]);
    assert!(!tree.contains("a.txt"), "deletion not committed: {tree}");
    assert!(tree.contains("b.txt"));

    // And it keeps working afterwards.
    write(&p.join("b.txt"), "b2");
    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "edit b"]);
    env.side(&p, &["auto"]);
    assert_eq!(env.side_git(&["show", "HEAD:b.txt"]), "b2");
}

#[test]
fn auto_picks_up_new_and_deleted_files_in_tracked_dir() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("notes/one.md"), "1");
    env.side(&p, &["add", "notes"]);
    env.side(&p, &["commit", "-m", "base"]);

    fs::remove_file(p.join("notes/one.md")).unwrap();
    write(&p.join("notes/two.md"), "2");
    env.git(&p, &["commit", "-q", "--allow-empty", "-m", "rotate"]);
    env.side(&p, &["auto"]);

    let tree = env.side_git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert!(!tree.contains("notes/one.md"), "{tree}");
    assert!(tree.contains("notes/two.md"), "{tree}");
}
