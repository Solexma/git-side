mod common;

use common::{TestEnv, write};

#[test]
fn add_of_tracked_path_restages_it() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("a.txt"), "v1");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "v1"]);

    write(&p.join("a.txt"), "v2");
    env.side(&p, &["add", "a.txt"]);
    env.side(&p, &["commit", "-m", "v2"]);

    assert_eq!(env.side_git(&["show", "HEAD:a.txt"]), "v2");
}

#[test]
fn add_inside_tracked_dir_does_not_duplicate_list_entry() {
    let env = TestEnv::new();
    let p = env.project("p");
    write(&p.join("docs/a.md"), "a");
    env.side(&p, &["add", "docs"]);
    env.side(&p, &["commit", "-m", "base"]);

    write(&p.join("docs/specs/b.md"), "b");
    env.side(&p, &["add", "docs/specs/b.md"]);
    env.side(&p, &["commit", "-m", "spec"]);

    assert_eq!(env.side_git(&["show", "HEAD:.side-tracked"]), "docs");
    assert_eq!(env.side_git(&["show", "HEAD:docs/specs/b.md"]), "b");
}
