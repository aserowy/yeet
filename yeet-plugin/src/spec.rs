use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub url: String,
    pub name: Option<String>,
    pub branch: Option<String>,
    pub version: Option<String>,
    pub dependencies: Vec<PluginSpec>,
    #[serde(default)]
    pub help_pages: Vec<PluginHelpPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHelpPage {
    pub name: String,
    pub content: String,
}

/// Discover help pages from a plugin's `docs/help/*.md` directory.
pub fn discover_help_pages(plugin_dir: &Path) -> Vec<PluginHelpPage> {
    let help_dir = plugin_dir.join("docs").join("help");
    if !help_dir.is_dir() {
        return Vec::new();
    }

    let entries = match std::fs::read_dir(&help_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut pages = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        pages.push(PluginHelpPage { name, content });
    }

    pages
}
