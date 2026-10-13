# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Going forward, entries are maintained by [release-please](https://github.com/googleapis/release-please) from conventional commit messages.

## [0.4.0](https://github.com/Solexma/git-side/compare/v0.3.0...v0.4.0) (2026-10-13)


### Features

* per-worktree side state for linked git worktrees ([462c4df](https://github.com/Solexma/git-side/commit/462c4df14d6cd60ab2da14baefda5906186e89e7))


### Bug Fixes

* address review findings in worktree support ([7c54b2e](https://github.com/Solexma/git-side/commit/7c54b2e72a15370efcc697d90e39c5971b3116b6))
* create side repos on main and migrate legacy default branches ([f23c8a9](https://github.com/Solexma/git-side/commit/f23c8a9b6ed8ed285a318196e12438b793c23fd8))
* hide main project files from git side status ([52d5d21](https://github.com/Solexma/git-side/commit/52d5d21b1317b8e92e20a165e4089f17ddbb3941))
* install hooks where git actually runs them ([6e75c6d](https://github.com/Solexma/git-side/commit/6e75c6d8ba5a8ad8a5a0478836ee762e145f6633))
* keep .side-tracked out of the project work-tree ([be32ba6](https://github.com/Solexma/git-side/commit/be32ba68efc4f2d7c4f9cf6307ceb10e726d796c))
* make add re-stage tracked paths instead of failing ([1a4d80e](https://github.com/Solexma/git-side/commit/1a4d80e16186e040ae89a3ab2f3791dc53e3d188))
* never record nested repositories as gitlinks ([12d12c1](https://github.com/Solexma/git-side/commit/12d12c1b1b46b86dcf9d3a388de76bc935824c13))
* resolve add/rm paths from the current directory and keep covered paths staged ([e58d100](https://github.com/Solexma/git-side/commit/e58d100c60085adf532ca0fd602513e32a813268))
* stop auto from aborting when a tracked path is deleted ([475e3ad](https://github.com/Solexma/git-side/commit/475e3adedd6042a0fc1d8855b4141ec969756fc6))


### Documentation

* describe merge target names ([3d92d29](https://github.com/Solexma/git-side/commit/3d92d29ba40cd0510e8f5f83d9ebb1d8c9a10c7c))
* document worktree support, checkout and merge ([f90daf0](https://github.com/Solexma/git-side/commit/f90daf096e3cfca9b7bef57d2e7e71a235709290))

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
