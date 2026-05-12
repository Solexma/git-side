# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Going forward, entries are maintained by [release-please](https://github.com/googleapis/release-please) from conventional commit messages.

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
