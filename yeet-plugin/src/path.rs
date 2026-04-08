use std::path::PathBuf;

pub fn resolve_plugin_data_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg).join("yeet").join("plugins"));
    }
    dirs::home_dir().map(|home| {
        home.join(".local")
            .join("share")
            .join("yeet")
            .join("plugins")
    })
}

pub fn resolve_lock_file_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("yeet").join("plugins.lock"));
    }
    dirs::home_dir().map(|home| home.join(".config").join("yeet").join("plugins.lock"))
}

pub fn url_to_storage_path(url: &str) -> Option<PathBuf> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git://"))
        .unwrap_or(url);

    let parts: Vec<&str> = without_scheme.split('/').collect();
    if parts.len() < 3 {
        return None;
    }

    let owner = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];
    Some(PathBuf::from(owner).join(repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_https_url() {
        let path = url_to_storage_path("https://github.com/aserowy/yeet-nord").unwrap();
        assert_eq!(path, PathBuf::from("aserowy").join("yeet-nord"));
    }

    #[test]
    fn github_https_url_with_git_suffix() {
        let path = url_to_storage_path("https://github.com/aserowy/yeet-nord.git").unwrap();
        assert_eq!(path, PathBuf::from("aserowy").join("yeet-nord"));
    }

    #[test]
    fn github_https_url_with_trailing_slash() {
        let path = url_to_storage_path("https://github.com/aserowy/yeet-nord/").unwrap();
        assert_eq!(path, PathBuf::from("aserowy").join("yeet-nord"));
    }

    #[test]
    fn invalid_url_too_few_parts() {
        assert!(url_to_storage_path("https://github.com").is_none());
    }

    #[test]
    fn resolve_data_path_without_xdg() {
        let path = resolve_plugin_data_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("yeet/plugins"));
    }

    #[test]
    fn resolve_lock_path_without_xdg() {
        let path = resolve_lock_file_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("yeet/plugins.lock"));
    }
}
