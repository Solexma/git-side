use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use colored::Colorize;

use crate::error::{Error, Result};
use crate::git;

const HOOK_MARKER_START: &str = "# >>> git-side auto >>>";
const HOOK_MARKER_END: &str = "# <<< git-side auto <<<";
const HOOK_CONTENT: &str = r"
# Auto-sync side-tracked files
git side auto
";

/// Get the path to a git hook.
fn hook_path(hook_name: &str) -> Result<PathBuf> {
    let git_dir = git::git_dir()?;
    Ok(git_dir.join("hooks").join(hook_name))
}

/// Check if our hook is already installed.
fn is_installed(hook_name: &str) -> Result<bool> {
    let path = hook_path(hook_name)?;
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&path).map_err(|e| Error::ReadFile {
        path: path.clone(),
        source: e,
    })?;

    Ok(content.contains(HOOK_MARKER_START))
}

/// Install the git-side hook.
///
/// # Errors
///
/// Returns an error if the hook is already installed or if file operations fail.
pub fn install(hook_name: &str) -> Result<()> {
    if is_installed(hook_name)? {
        return Err(Error::HookAlreadyInstalled(hook_name.to_string()));
    }

    let path = hook_path(hook_name)?;

    // Ensure hooks directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Read existing content or create new
    let existing = if path.exists() {
        fs::read_to_string(&path).map_err(|e| Error::ReadFile {
            path: path.clone(),
            source: e,
        })?
    } else {
        "#!/bin/sh\n".to_string()
    };

    // Append our hook
    let new_content = format!(
        "{existing}\n{HOOK_MARKER_START}{HOOK_CONTENT}{HOOK_MARKER_END}\n"
    );

    fs::write(&path, new_content).map_err(|e| Error::WriteFile {
        path: path.clone(),
        source: e,
    })?;

    // Make executable (Unix only - Windows doesn't need this)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path)
            .map_err(|e| Error::ReadFile {
                path: path.clone(),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).map_err(|e| Error::WriteFile {
            path: path.clone(),
            source: e,
        })?;
    }

    println!(
        "{} {} hook installed",
        "Done.".green().bold(),
        hook_name.cyan()
    );

    Ok(())
}

/// Uninstall the git-side hook.
///
/// # Errors
///
/// Returns an error if the hook is not installed or if file operations fail.
pub fn uninstall(hook_name: &str) -> Result<()> {
    if !is_installed(hook_name)? {
        return Err(Error::HookNotInstalled(hook_name.to_string()));
    }

    let path = hook_path(hook_name)?;

    let content = fs::read_to_string(&path).map_err(|e| Error::ReadFile {
        path: path.clone(),
        source: e,
    })?;

    // Remove our section
    let mut new_lines = Vec::new();
    let mut in_our_section = false;

    for line in content.lines() {
        if line.contains(HOOK_MARKER_START) {
            in_our_section = true;
            continue;
        }
        if line.contains(HOOK_MARKER_END) {
            in_our_section = false;
            continue;
        }
        if !in_our_section {
            new_lines.push(line);
        }
    }

    let new_content = new_lines.join("\n");

    // Check if only shebang remains
    let trimmed = new_content.trim();
    if trimmed.is_empty() || trimmed == "#!/bin/sh" || trimmed == "#!/bin/bash" {
        // Remove the file entirely
        fs::remove_file(&path).map_err(|e| Error::WriteFile {
            path: path.clone(),
            source: e,
        })?;
    } else {
        fs::write(&path, new_content).map_err(|e| Error::WriteFile {
            path: path.clone(),
            source: e,
        })?;
    }

    println!(
        "{} {} hook removed",
        "Done.".green().bold(),
        hook_name.cyan()
    );

    Ok(())
}
