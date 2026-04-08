use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub url: String,
    pub name: Option<String>,
    pub branch: Option<String>,
    pub version: Option<String>,
    pub dependencies: Vec<PluginSpec>,
}
