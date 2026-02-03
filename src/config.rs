use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Get the config directory path (~/.config/git-side/).
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("git-side")
}

/// Get the cache file path (~/.config/git-side/cache).
fn cache_file() -> PathBuf {
    config_dir().join("cache")
}

/// Get the paths file path (~/.config/git-side/paths).
fn paths_file() -> PathBuf {
    config_dir().join("paths")
}

/// Ensure the config directory exists.
fn ensure_config_dir() -> Result<()> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| Error::CreateDir {
            path: dir,
            source: e,
        })?;
    }
    Ok(())
}

/// Read a key=value file into a `HashMap`.
fn read_kv_file(path: &Path) -> Result<HashMap<String, String>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path).map_err(|e| Error::ReadFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    Ok(map)
}

/// Write a `HashMap` to a key=value file.
fn write_kv_file(path: &Path, map: &HashMap<String, String>) -> Result<()> {
    ensure_config_dir()?;

    let content: String = map
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(path, content).map_err(|e| Error::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Hash a path to a 16-character hex string.
#[must_use]
pub fn hash_path(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Cache: lookup root SHA by hashed repo path.
///
/// # Errors
///
/// Returns an error if the cache file cannot be read.
pub fn cache_lookup(path_hash: &str) -> Result<Option<String>> {
    let map = read_kv_file(&cache_file())?;
    Ok(map.get(path_hash).cloned())
}

/// Cache: store root SHA for hashed repo path.
///
/// # Errors
///
/// Returns an error if the cache file cannot be written.
pub fn cache_store(path_hash: &str, root_sha: &str) -> Result<()> {
    let mut map = read_kv_file(&cache_file())?;
    map.insert(path_hash.to_string(), root_sha.to_string());
    write_kv_file(&cache_file(), &map)
}

/// Paths: lookup custom base path by root SHA.
///
/// # Errors
///
/// Returns an error if the paths file cannot be read.
pub fn paths_lookup(root_sha: &str) -> Result<Option<PathBuf>> {
    let map = read_kv_file(&paths_file())?;
    Ok(map.get(root_sha).map(PathBuf::from))
}

/// Paths: store custom base path for root SHA.
///
/// # Errors
///
/// Returns an error if the paths file cannot be written.
pub fn paths_store(root_sha: &str, base_path: &Path) -> Result<()> {
    let mut map = read_kv_file(&paths_file())?;
    map.insert(
        root_sha.to_string(),
        base_path.to_string_lossy().to_string(),
    );
    write_kv_file(&paths_file(), &map)
}

/// Get the default base path for side repos.
#[must_use]
pub fn default_base_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("git-side")
}
