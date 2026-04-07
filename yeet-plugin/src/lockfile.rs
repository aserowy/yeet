use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockFile {
    #[serde(default)]
    pub plugins: BTreeMap<String, LockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub commit: String,
    pub sha256: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
}

impl LockFile {
    pub fn read_from(path: &Path) -> Result<Self, LockFileError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(LockFileError::Io)?;
        toml::from_str(&content).map_err(LockFileError::Parse)
    }

    pub fn write_to(&self, path: &Path) -> Result<(), LockFileError> {
        let content = toml::to_string_pretty(self).map_err(LockFileError::Serialize)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(LockFileError::Io)?;
        }
        std::fs::write(path, content).map_err(LockFileError::Io)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockFileError {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("failed to parse lock file: {0}")]
    Parse(toml::de::Error),
    #[error("failed to serialize lock file: {0}")]
    Serialize(toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plugins.lock");

        let mut lock = LockFile::default();
        lock.plugins.insert(
            "github.com/user/plugin".to_string(),
            LockEntry {
                commit: "abc123".to_string(),
                sha256: "def456".to_string(),
                branch: Some("main".to_string()),
                tag: Some("v1.0.0".to_string()),
            },
        );

        lock.write_to(&path).unwrap();
        let loaded = LockFile::read_from(&path).unwrap();

        assert_eq!(loaded.plugins.len(), 1);
        let entry = &loaded.plugins["github.com/user/plugin"];
        assert_eq!(entry.commit, "abc123");
        assert_eq!(entry.sha256, "def456");
        assert_eq!(entry.branch.as_deref(), Some("main"));
        assert_eq!(entry.tag.as_deref(), Some("v1.0.0"));
    }

    #[test]
    fn missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.lock");

        let lock = LockFile::read_from(&path).unwrap();
        assert!(lock.plugins.is_empty());
    }

    #[test]
    fn corrupt_file_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plugins.lock");
        std::fs::write(&path, "this is not valid toml [[[").unwrap();

        let result = LockFile::read_from(&path);
        assert!(result.is_err());
    }
}
