# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Going forward, entries are maintained by [release-please](https://github.com/googleapis/release-please) from conventional commit messages.

## [0.4.2](https://github.com/Solexma/git-side/compare/v0.4.1...v0.4.2) (2026-10-13)


### Bug Fixes

* report side files checked out while setting up a worktree ([a3b5951](https://github.com/Solexma/git-side/commit/a3b5951d84b8eae879ecf093a2127326e6e06503))
* warn about unmerged side commits from inside the worktree ([e331a84](https://github.com/Solexma/git-side/commit/e331a842f53b7a4f76c11b9e3936bb0fa1b05798))

## [0.4.1](https://github.com/Solexma/git-side/compare/v0.4.0...v0.4.1) (2026-10-13)

Patch release for packagers: the integration tests now pass in build environments that override git's configuration, such as makepkg (`GIT_CONFIG_GLOBAL=/dev/null`), so source-built packages like the AUR's can run `cargo test` in `check()`. No change to the `git-side` binary itself.


### Bug Fixes

* make integration tests independent of the caller's git config ([898dcf8](https://github.com/Solexma/git-side/commit/898dcf8f7a3179b1244f45e2ef33c17f4bf84f50))

## [0.4.0](https://github.com/Solexma/git-side/compare/v0.3.0...v0.4.0) (2026-10-13)

This release adds **support for git worktrees** and fixes several bugs that could silently lose or corrupt side-repo data. Upgrading is recommended for everyone, especially if you use `git worktree` or tools that create worktrees for you, such as Claude Code under `.claude/worktrees/`.

### Upgrade notes

The first command you run after upgrading updates your existing side repos automatically:

- **The side repo's branch is renamed to `main`** if it was created on another branch (usually `master`, from git's default). `git side push` only ever pushed `main`, so on a default git setup it failed with `src refspec main does not match any`. After upgrading, `git side push` creates `main` on your remote. An old `master` branch there is left alone until you delete it.
- **`.side-tracked` no longer appears in your project.** Older versions of `git side pull` wrote a stray `.side-tracked` file into the project directory. You can delete it safely. The tracked list is versioned in the side repo.
- **Broken submodule-style entries get cleaned up.** If a tracked directory contained a git worktree or repository, older versions recorded it as a broken submodule-style entry (a "gitlink"). The next `git side auto` removes it.

### Features

**Worktree support.** Each linked worktree now gets its own side state: its own index and tracked list, on a side branch `wt/<name>` that starts from `main`.

- The first `git side` command in a new worktree checks out the side files it's missing (`CLAUDE.md`, notes, configs…). Files that already exist are never overwritten.
- New `git side checkout` checks out side files missing from the current worktree.
- New `git side merge <main|wt/<name>|name>` brings side changes from one worktree into another. Resolve conflicts the git way, then run `git side merge --continue` or `--abort`.
- `git side info` lists worktree branches that haven't been merged into `main` yet.
- When you remove a worktree, git cleans up its side state. Its `wt/<name>` branch is kept, so unmerged work is never lost.
- `push` and `pull` only run in the main worktree. `auto` in a worktree commits to its side branch without pushing.

### Bug fixes

- **`auto` in a fresh worktree deleted your side files.** All worktrees shared one side index. Running `auto` from a worktree (for example through the post-commit hook) recorded every side file it didn't have as deleted.
- **`auto` broke forever once a tracked file was deleted.** Every later run failed with `pathspec did not match`.
- **Deleted files with non-ASCII names (e.g. `café.md`) were never removed** from the side repo.
- **`git side add` on an already-tracked path failed** with `path already tracked`. Since `commit` doesn't stage changes, there was no way to commit an edit except `auto`. `add` now stages the current content, like `git add`.
- **Paths given to `add` and `rm` ignored the current directory.** They are now resolved relative to where you run the command, like git does. Paths outside the repository are rejected.
- **`git side rm` on an entry inside a tracked directory unstaged the file**, even though the directory still covers it. Adding a directory now also absorbs entries already inside it, so duplicates no longer pile up.
- **Nested worktrees and repositories inside tracked directories** were committed as gitlinks. They are now excluded.
- **`git side hook install` from a worktree installed a hook that never ran**, and `core.hooksPath` (husky, lefthook…) was ignored. Hooks now go where git actually runs them.
- **`git side status` listed the whole project as untracked** and always reported `.side-tracked` as deleted.

### Other

- First integration test suite: 38 tests that run the real binary against throwaway repositories.
- README: new "Worktrees" section.

## [0.3.0](https://github.com/Solexma/git-side/releases/tag/v0.3.0) (2026-05-12)

### Features

- Add `git side ls-files` command for listing tracked files (passthrough to `git ls-files`)

### Bug Fixes

- Allow hyphen-prefixed flags in `git side ls-files` (e.g. `--modified`, `--error-unmatch`) without needing the `--` separator
- Allow hyphen-prefixed flags in `git side log` and `git side remote` (e.g. `git side log --oneline`, `git side remote -v`) without needing `--`

### Documentation

- Add pre-built binary installation instructions to README

## [0.2.3](https://github.com/Solexma/git-side/releases/tag/v0.2.3) (2026-02-20)

### Features

- `git side auto` now auto-pushes to the configured remote after committing

### Documentation

- Document auto-push behavior in README

## [0.2.2](https://github.com/Solexma/git-side/releases/tag/v0.2.2) (2026-02-18)

### Bug Fixes

- Add Windows compatibility for hook installation (#1)

## [0.2.1](https://github.com/Solexma/git-side/releases/tag/v0.2.1) (2026-02-03)

### Bug Fixes

- Adjust docs for the platform-specific config directory

## [0.2.0](https://github.com/Solexma/git-side/releases/tag/v0.2.0) (2026-02-03)

### Features

- Initial implementation: `add`, `rm`, `status`, `commit`, `log`, `auto`, `init`, `hook`
- Remote, push, and pull commands for side repo synchronization
- Self-aware side repo: `.side-tracked` is now versioned inside the side repo itself using git plumbing
- `git side info` command

### Bug Fixes

- Clear inherited git environment variables in hooks to prevent leaking to main repo
- Specify `git-dir` and `work-tree` explicitly for side commands
- Use correct working tree when git-side is installed as a binary
