use std::path::Path;
use std::sync::atomic::AtomicBool;

use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git operation failed: {0}")]
    Gix(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no matching reference found")]
    NoMatchingRef,
}

pub fn clone_at_ref(url: &str, target: &Path, reference: &str) -> Result<(), GitError> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(GitError::Io)?;
    }

    let mut prepare = prepare_clone_no_credentials(url, target)?;

    let (mut checkout, _outcome) = prepare
        .fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))
        .map_err(gix_err)?;

    let (repo, _) = checkout
        .main_worktree(gix::progress::Discard, &AtomicBool::new(false))
        .map_err(gix_err)?;

    checkout_ref(&repo, reference)?;

    Ok(())
}

pub fn clone_branch_head(url: &str, target: &Path, branch: Option<&str>) -> Result<(), GitError> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(GitError::Io)?;
    }

    let mut prepare = prepare_clone_no_credentials(url, target)?;

    if let Some(branch) = branch {
        prepare = prepare.with_ref_name(Some(branch)).map_err(gix_err)?;
    }

    let (mut checkout, _outcome) = prepare
        .fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))
        .map_err(gix_err)?;

    checkout
        .main_worktree(gix::progress::Discard, &AtomicBool::new(false))
        .map_err(gix_err)?;

    Ok(())
}

pub fn fetch_and_checkout(repo_path: &Path, commit_id: &str) -> Result<(), GitError> {
    let repo = open_no_credentials(repo_path)?;

    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .ok_or_else(|| GitError::Gix("no default remote found".to_string()))?
        .map_err(gix_err)?;

    remote
        .connect(gix::remote::Direction::Fetch)
        .map_err(gix_err)?
        .prepare_fetch(gix::progress::Discard, Default::default())
        .map_err(gix_err)?
        .receive(gix::progress::Discard, &AtomicBool::new(false))
        .map_err(gix_err)?;

    checkout_ref(&repo, commit_id)?;

    Ok(())
}

pub fn list_remote_tags(repo_path: &Path, url: &str) -> Result<Vec<String>, GitError> {
    let repo = open_no_credentials(repo_path)?;

    let remote = repo.remote_at(url).map_err(gix_err)?;

    let connection = remote
        .connect(gix::remote::Direction::Fetch)
        .map_err(gix_err)?;

    let (ref_map, _handshake) = connection
        .ref_map(gix::progress::Discard, Default::default())
        .map_err(gix_err)?;

    let tags: Vec<String> = ref_map
        .remote_refs
        .iter()
        .filter_map(|r| {
            let name = match r {
                gix::protocol::handshake::Ref::Direct { full_ref_name, .. }
                | gix::protocol::handshake::Ref::Peeled { full_ref_name, .. } => {
                    full_ref_name.to_string()
                }
                _ => return None,
            };

            name.strip_prefix("refs/tags/").map(|tag| tag.to_string())
        })
        .collect();

    Ok(tags)
}

pub fn resolve_remote_head(repo_path: &Path, url: &str) -> Result<String, GitError> {
    let repo = open_no_credentials(repo_path)?;

    let remote = repo.remote_at(url).map_err(gix_err)?;

    let connection = remote
        .connect(gix::remote::Direction::Fetch)
        .map_err(gix_err)?;

    let (ref_map, _handshake) = connection
        .ref_map(gix::progress::Discard, Default::default())
        .map_err(gix_err)?;

    for r in &ref_map.remote_refs {
        if let gix::protocol::handshake::Ref::Symbolic {
            full_ref_name,
            target,
            ..
        } = r
        {
            if *full_ref_name == "HEAD" {
                return target
                    .to_string()
                    .strip_prefix("refs/heads/")
                    .map(|s| s.to_string())
                    .ok_or(GitError::NoMatchingRef);
            }
        }
    }

    Err(GitError::NoMatchingRef)
}

pub fn compute_tree_sha256(repo_path: &Path) -> Result<String, GitError> {
    let mut hasher = Sha256::new();
    hash_directory(&mut hasher, repo_path, repo_path)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_directory(hasher: &mut Sha256, base: &Path, dir: &Path) -> Result<(), GitError> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);

        if relative.starts_with(".git") {
            continue;
        }

        let metadata = entry.metadata()?;
        hasher.update(relative.to_string_lossy().as_bytes());

        if metadata.is_file() {
            let content = std::fs::read(&path)?;
            hasher.update(&content);
        } else if metadata.is_dir() {
            hash_directory(hasher, base, &path)?;
        }
    }

    Ok(())
}

fn checkout_ref(repo: &gix::Repository, reference: &str) -> Result<(), GitError> {
    let id = repo.rev_parse_single(reference).map_err(gix_err)?;

    let tree = id
        .object()
        .map_err(gix_err)?
        .peel_to_tree()
        .map_err(gix_err)?;

    let index = repo.index_from_tree(&tree.id).map_err(gix_err)?;

    let opts = gix::worktree::state::checkout::Options {
        overwrite_existing: true,
        ..Default::default()
    };

    gix::worktree::state::checkout(
        &mut index.into(),
        repo.workdir()
            .ok_or_else(|| GitError::Gix("bare repository has no worktree".to_string()))?,
        repo.objects.clone().into_arc().map_err(gix_err)?,
        &gix::progress::Discard,
        &gix::progress::Discard,
        &AtomicBool::new(false),
        opts,
    )
    .map_err(gix_err)?;

    Ok(())
}

pub fn open_no_credentials(path: &Path) -> Result<gix::Repository, GitError> {
    let opts = gix::open::Options::default().config_overrides([
        "credential.helper=",
        "gitoxide.credentials.terminalPrompt=false",
    ]);
    gix::open_opts(path, opts).map_err(gix_err)
}

pub fn prepare_clone_no_credentials(
    url: &str,
    target: &Path,
) -> Result<gix::clone::PrepareFetch, GitError> {
    Ok(gix::prepare_clone(url, target)
        .map_err(gix_err)?
        .with_in_memory_config_overrides([
            "credential.helper=",
            "gitoxide.credentials.terminalPrompt=false",
        ]))
}

fn gix_err(e: impl std::fmt::Display) -> GitError {
    GitError::Gix(e.to_string())
}
