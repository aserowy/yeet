use std::path::Path;

use crate::git::{self, GitError};
use crate::lockfile::{LockEntry, LockFile, LockFileError};
use crate::path::url_to_storage_path;
use crate::spec::PluginSpec;
use crate::sync::{cleanup_unregistered, collect_all_specs, lock_key_for_url, PluginSyncError};
use crate::version::{filter_tags_by_range, parse_version_range};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("lock file error: {0}")]
    LockFile(#[from] LockFileError),
}

pub struct UpdateResult {
    pub updated: Vec<String>,
    pub errors: Vec<PluginSyncError>,
    pub removed: Vec<String>,
}

pub fn update(
    specs: &[PluginSpec],
    lock_file_path: &Path,
    data_path: &Path,
) -> Result<UpdateResult, UpdateError> {
    let mut lock = LockFile::read_from(lock_file_path)?;

    let removed = cleanup_unregistered(specs, &mut lock, data_path);

    let mut updated = Vec::new();
    let mut errors = Vec::new();

    let all_specs = collect_all_specs(specs);

    for spec in &all_specs {
        match update_single_plugin(spec, &mut lock, data_path) {
            Ok(()) => updated.push(spec.url.clone()),
            Err(e) => errors.push(PluginSyncError {
                url: spec.url.clone(),
                error: e.to_string(),
            }),
        }
    }

    lock.write_to(lock_file_path)?;

    Ok(UpdateResult {
        updated,
        errors,
        removed,
    })
}

fn update_single_plugin(
    spec: &PluginSpec,
    lock: &mut LockFile,
    data_path: &Path,
) -> Result<(), GitError> {
    let storage_path = url_to_storage_path(&spec.url)
        .ok_or_else(|| GitError::Gix(format!("invalid URL: {}", spec.url)))?;
    let plugin_path = data_path.join(&storage_path);

    let (commit, tag) = if let Some(version_constraint) = &spec.version {
        resolve_tagged_version(spec, &plugin_path, version_constraint)?
    } else {
        resolve_branch_head(spec, &plugin_path)?
    };

    let sha256 = git::compute_tree_sha256(&plugin_path)?;

    let key = lock_key_for_url(&spec.url);
    lock.plugins.insert(
        key,
        LockEntry {
            commit,
            sha256,
            branch: spec.branch.clone(),
            tag,
        },
    );

    Ok(())
}

fn resolve_tagged_version(
    spec: &PluginSpec,
    plugin_path: &Path,
    version_constraint: &str,
) -> Result<(String, Option<String>), GitError> {
    let range = parse_version_range(version_constraint)
        .map_err(|e| GitError::Gix(format!("invalid version constraint: {}", e)))?;

    ensure_cloned(spec, plugin_path)?;

    let tags = git::list_remote_tags(plugin_path, &spec.url)?;
    let matching = filter_tags_by_range(&tags, &range);

    let (tag_name, _version) = matching.first().ok_or_else(|| {
        GitError::Gix(format!(
            "no tag matching '{}' found for {}",
            version_constraint, spec.url
        ))
    })?;

    git::fetch_and_checkout(plugin_path, tag_name)?;

    let repo = gix::open(plugin_path).map_err(|e| GitError::Gix(e.to_string()))?;
    let id = repo
        .rev_parse_single(tag_name.as_bytes())
        .map_err(|e| GitError::Gix(e.to_string()))?;

    Ok((id.to_string(), Some(tag_name.to_string())))
}

fn resolve_branch_head(
    spec: &PluginSpec,
    plugin_path: &Path,
) -> Result<(String, Option<String>), GitError> {
    if plugin_path.exists() {
        git::fetch_and_checkout(
            plugin_path,
            &format!("origin/{}", spec.branch.as_deref().unwrap_or("HEAD")),
        )?;
    } else {
        git::clone_branch_head(&spec.url, plugin_path, spec.branch.as_deref())?;
    }

    let repo = gix::open(plugin_path).map_err(|e| GitError::Gix(e.to_string()))?;
    let head = repo
        .head_commit()
        .map_err(|e| GitError::Gix(e.to_string()))?;

    Ok((head.id.to_string(), None))
}

fn ensure_cloned(spec: &PluginSpec, plugin_path: &Path) -> Result<(), GitError> {
    if !plugin_path.exists() {
        git::clone_branch_head(&spec.url, plugin_path, spec.branch.as_deref())?;
    }
    Ok(())
}
