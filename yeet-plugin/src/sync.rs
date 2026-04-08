use std::path::Path;
use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::git::{self, GitError};
use crate::lockfile::{LockEntry, LockFile, LockFileError};
use crate::path::url_to_storage_path;
use crate::spec::PluginSpec;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("lock file error: {0}")]
    LockFile(#[from] LockFileError),
    #[error("no lock file found, run :pluginupdate first")]
    NoLockFile,
    #[error("plugin errors: {0:?}")]
    PluginErrors(Vec<PluginSyncError>),
}

#[derive(Debug)]
pub struct PluginSyncError {
    pub url: String,
    pub error: String,
}

pub struct SyncResult {
    pub synced: Vec<String>,
    pub errors: Vec<PluginSyncError>,
    pub removed: Vec<String>,
}

enum PluginOutcome {
    Synced(String),
    Error(PluginSyncError),
}

pub async fn sync(
    specs: &[PluginSpec],
    lock_file_path: &Path,
    data_path: &Path,
    concurrency: usize,
) -> Result<SyncResult, SyncError> {
    if !lock_file_path.exists() {
        return Err(SyncError::NoLockFile);
    }

    let mut lock = LockFile::read_from(lock_file_path)?;
    let removed = cleanup_unregistered(specs, &mut lock, data_path);

    let all_specs = collect_all_specs(specs);
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut handles = Vec::new();
    for spec in all_specs {
        let key = lock_key_for_url(&spec.url);
        let entry = match lock.plugins.get(&key).cloned() {
            Some(entry) => entry,
            None => {
                let url = spec.url.clone();
                handles.push(tokio::spawn(async move {
                    PluginOutcome::Error(PluginSyncError {
                        url,
                        error: "not in lock file, run :pluginupdate".to_string(),
                    })
                }));
                continue;
            }
        };

        let sem = semaphore.clone();
        let data_path = data_path.to_path_buf();
        let url = spec.url.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let url_for_result = url.clone();
            let result =
                tokio::task::spawn_blocking(move || sync_single_plugin(&spec, &entry, &data_path))
                    .await
                    .expect("blocking task panicked");

            match result {
                Ok(()) => PluginOutcome::Synced(url_for_result),
                Err(e) => PluginOutcome::Error(PluginSyncError {
                    url: url_for_result,
                    error: e.to_string(),
                }),
            }
        }));
    }

    let mut synced = Vec::new();
    let mut errors = Vec::new();
    for handle in handles {
        match handle.await.expect("task panicked") {
            PluginOutcome::Synced(url) => synced.push(url),
            PluginOutcome::Error(e) => errors.push(e),
        }
    }

    if !removed.is_empty() {
        lock.write_to(lock_file_path)?;
    }

    Ok(SyncResult {
        synced,
        errors,
        removed,
    })
}

fn sync_single_plugin(
    spec: &PluginSpec,
    entry: &LockEntry,
    data_path: &Path,
) -> Result<(), GitError> {
    let storage_path = url_to_storage_path(&spec.url)
        .ok_or_else(|| GitError::Gix(format!("invalid URL: {}", spec.url)))?;
    let plugin_path = data_path.join(storage_path);

    if plugin_path.exists() {
        git::fetch_and_checkout(&plugin_path, &entry.commit)?;
    } else {
        git::clone_at_ref(&spec.url, &plugin_path, &entry.commit)?;
    }

    let hash = git::compute_tree_sha256(&plugin_path)?;
    if hash != entry.sha256 {
        return Err(GitError::Gix(format!(
            "integrity check failed: expected {}, got {}",
            entry.sha256, hash
        )));
    }

    Ok(())
}

pub fn cleanup_unregistered(
    specs: &[PluginSpec],
    lock: &mut LockFile,
    data_path: &Path,
) -> Vec<String> {
    let all_specs = collect_all_specs(specs);
    let registered_keys: std::collections::HashSet<String> =
        all_specs.iter().map(|s| lock_key_for_url(&s.url)).collect();

    let to_remove: Vec<String> = lock
        .plugins
        .keys()
        .filter(|k| !registered_keys.contains(*k))
        .cloned()
        .collect();

    for key in &to_remove {
        lock.plugins.remove(key);

        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2];
            let repo = parts[parts.len() - 1];
            let dir = data_path.join(owner).join(repo);
            if dir.exists() {
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(data_path) {
        for owner_entry in entries.flatten() {
            if !owner_entry.path().is_dir() {
                continue;
            }
            if let Ok(repo_entries) = std::fs::read_dir(owner_entry.path()) {
                for repo_entry in repo_entries.flatten() {
                    if !repo_entry.path().is_dir() {
                        continue;
                    }
                    let owner = owner_entry.file_name().to_string_lossy().to_string();
                    let repo = repo_entry.file_name().to_string_lossy().to_string();
                    let key_candidate = format!("{}/{}", owner, repo);
                    if !registered_keys.iter().any(|k| k.ends_with(&key_candidate)) {
                        let _ = std::fs::remove_dir_all(repo_entry.path());
                    }
                }
            }
        }
    }

    to_remove
}

pub fn collect_all_specs(specs: &[PluginSpec]) -> Vec<PluginSpec> {
    let mut all = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for spec in specs {
        for dep in &spec.dependencies {
            if seen.insert(dep.url.clone()) {
                all.push(dep.clone());
            }
        }
        if seen.insert(spec.url.clone()) {
            all.push(PluginSpec {
                url: spec.url.clone(),
                name: spec.name.clone(),
                branch: spec.branch.clone(),
                version: spec.version.clone(),
                dependencies: Vec::new(),
            });
        }
    }

    all
}

pub fn lock_key_for_url(url: &str) -> String {
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git://"))
        .unwrap_or(url)
        .to_string()
}
