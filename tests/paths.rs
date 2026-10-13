mod common;

use common::{TestEnv, write};

#[test]
fn add_and_rm_resolve_paths_from_subdirectory() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("sub/notes.md"), "n");
    write(&p.join("top.md"), "t");
    let sub = p.join("sub");

    env.side(&sub, &["add", "notes.md"]);
    env.side(&sub, &["add", "../top.md"]);
    env.side(&sub, &["commit", "-m", "base"]);
    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "sub/notes.md\ntop.md");

    env.side(&sub, &["rm", "./notes.md"]);
    env.side(&sub, &["commit", "-m", "rm"]);
    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "top.md");
}

#[test]
fn add_accepts_absolute_path_and_rejects_outside_repo() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "a");
    write(&env.root.join("outside.txt"), "o");

    env.side(&p, &["add", p.join("a.txt").to_str().unwrap()]);
    assert_eq!(env.ls_files(&p), vec![".side-tracked", "a.txt"]);

    let out = env.side_raw(&p, &["add", env.root.join("outside.txt").to_str().unwrap()]);
    assert!(!out.status.success());
    let out = env.side_raw(&p, &["add", "../outside.txt"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("outside"));
}

#[test]
fn rm_of_path_covered_by_tracked_dir_keeps_it_staged() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("docs/a.md"), "a");
    env.side(&p, &["add", "docs"]);
    // Redundant entry, as older versions recorded it.
    env.set_tracked_list(&p, "docs\ndocs/a.md");
    env.side(&p, &["commit", "-m", "base"]);
    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "docs\ndocs/a.md");

    env.side(&p, &["rm", "docs/a.md"]);
    env.side(&p, &["commit", "-m", "dedupe"]);

    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "docs");
    assert_eq!(env.side_git(&["show", "HEAD:docs/a.md"]), "a");
}

#[test]
fn adding_a_directory_absorbs_entries_inside_it() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("docs/a.md"), "a");
    write(&p.join("docs/sub/b.md"), "b");
    env.side(&p, &["add", "docs/a.md"]);
    env.side(&p, &["add", "docs/sub/b.md"]);

    env.side(&p, &["add", "docs"]);
    env.side(&p, &["commit", "-m", "base"]);

    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "docs");
}
