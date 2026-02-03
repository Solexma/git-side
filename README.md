# git-side

`git-side` is a Git subcommand that allows you to version files and directories that **should not live in the main repo**, using a separate, per-project **bare Git repo**, completely invisible to the original repo.

It is designed for local-only state, tooling artifacts, and contextual files that must be versioned but must not pollute the primary project history.

## Why git-side exists

In real projects there are files that:

- are intentionally excluded via `.gitignore` or global ignore rules
- are local, contextual, or environment-specific
- contain tooling state (AI prompts, build metadata, local configs)
- must be versioned, but **not in the main repo**

Examples:
- `BACKLOG.md`, `TODO.md`, `NOTES.md` — personal project notes
- `.vscode/launch.json` — local debug configs not shared with the team
- `docker-compose.override.yml` — local dev environment overrides
- `scratch/` — throwaway experiments and prototypes
- `.claude/`, `.cursor/rules/` — when the team prefers to keep them out

Git *can* technically track these files, but **you should not** do it in the main repo.

`git-side` solves this by introducing a **side repo**:
- per project
- Git-native
- invisible to the main repo
- using a bare repo, vcsh-style

> **git-side is NOT for secrets.** Files like `.env`, API keys, tokens, and credentials should **never** be committed to any repo — not the main one, not the side one. Use a secrets manager. No exceptions.

## Core concepts

### Side repo

For each Git project, `git-side` creates (lazily) a dedicated **bare repo** stored outside the project directory:

```bash
~/.local/share/git-side/<initial-commit-sha>/
```

The **initial commit SHA** (`git rev-list --max-parents=0 HEAD`) is used as the project identifier. It is immutable, exists in every repo, and is stable across clones regardless of filesystem location or remote URL.

You can set a custom base path per project:

```bash
git side init --path /mnt/external/side-repos/
```

This stores the mapping in `~/.config/git-side/paths` — no changes to the main repo.

The project directory itself is used as the **work-tree**.

The main repo is never modified:
- no submodules
- no config changes
- no hooks (unless you opt-in with `git side hook install`)
- no metadata files

`git side hook install` adds a local hook to automate `git side auto`. Since `.git/hooks/` is not tracked by Git, this remains invisible to the repo and other clones.

Supported hooks include:
- `post-commit` — sync after every commit (default)
- `pre-push` — sync before pushing
- `post-merge` — sync after pulling/merging

### Directories are semantic containers

In `git-side`, directories are treated as **semantic containers**.

If you track a directory, `git-side` assumes responsibility for **everything inside it**:

- new files
- deleted files
- renamed files
- nested directories

This behavior is implemented by the tool itself, not delegated to Git defaults.

### Ignore rules are bypassed by design

`git-side` always stages files using:

```bash
git add -f
```

This means:

- .gitignore
- global ignore (core.excludesFile)
- system ignore rules

are intentionally ignored for side-tracked paths.

This is a feature, not a workaround.

## Installation

### From source (Rust)

```bash
cargo build --release
install -m 755 target/release/git-side ~/.local/bin/git-side
```

Make sure `~/.local/bin` is in your `$PATH`.

## Usage

`git-side` is a Git subcommand. Once installed, it is invoked as:

```bash
git side <command> [<args>]
```

### Commands

```bash
git side add <path>                    # track file or directory (forced, bypasses gitignore)
git side rm <path>                     # untrack path from side repo
git side status                        # show side repo status
git side commit -m "msg"               # commit in side repo
git side log                           # show side repo history
git side auto                          # sync side-tracked paths and commit using last main repo message
git side init --path <dir>             # set custom base path for this project's side repo
git side hook install [--on <hook>]    # install git hook to run auto (default: post-commit)
git side hook uninstall [--on <hook>]  # remove git hook
git side info                          # show info about git-side and current project
git side remote [<args>]               # manage remotes (pass-through to git remote)
git side push                          # push to origin/main (force, local wins)
git side pull                          # pull from origin/main (force, remote wins)
```

If you know Git, you already know `git-side`.

### Examples

```bash
# track some files
git side add BACKLOG.md
git side add scratch/

# commit to side repo
git side commit -m "Added personal backlog"

# or piggyback on the main repo's last commit message
git commit -m "Refactor parsing logic"
git side auto
```

Untracked files from the main project are hidden by default.

### Remote sync

```bash
# add a remote to your side repo
git side remote add origin git@github.com:user/project-side.git

# push (force, local always wins)
git side push

# pull (force, remote always wins)
git side pull

# list remotes
git side remote
```

Push and pull are intentionally simple and conflict-free:
- **push** uses `--force` — your local side repo always wins
- **pull** uses `fetch` + `reset --hard` — the remote always wins

This matches the "local-only state" philosophy. If you need merge semantics, you're probably tracking the wrong files.

## Design goals

- Git-native behavior
- No impact on the main repo
- Per-project isolation
- Deterministic and reproducible
- No dotfiles, no metadata in the project
- Works with existing Git workflows

## Non-goals

`git-side` intentionally does not:

- handle merge conflicts (push/pull are force operations)
- encrypt or secure files
- replace secrets managers
- act as a dotfiles manager
- integrate with the main repo history

**It is a local, explicit, opt-in tool.** Remote sync is supported but kept simple — no conflict resolution.

## Inspiration

The core model is inspired by:

- bare repos
- [vcsh](https://github.com/RichiH/vcsh) — manages multiple Git repos with `$HOME` as work-tree

But applied per project, not per `$HOME`.

## Author

Created by [MiPnamic](https://github.com/MiPnamic) for [Solexma](https://github.com/Solexma).

## License

MIT — see [LICENSE](LICENSE).
