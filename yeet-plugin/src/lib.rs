mod git;
mod lockfile;
mod path;
mod spec;
mod state;
pub mod sync;
pub mod update;
mod version;

pub use git::{
    clone_at_ref, clone_branch_head, compute_tree_sha256, fetch_and_checkout, list_remote_tags,
    resolve_remote_head, GitError,
};
pub use lockfile::{LockEntry, LockFile};
pub use path::{resolve_lock_file_path, resolve_plugin_data_path, url_to_storage_path};
pub use spec::PluginSpec;
pub use state::{PluginState, PluginStatus};
pub use version::{deduplicate_dependencies, filter_tags_by_range, parse_version_range};
