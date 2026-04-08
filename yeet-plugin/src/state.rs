#[derive(Debug, Clone)]
pub struct PluginState {
    pub url: String,
    pub status: PluginStatus,
    pub error_message: Option<String>,
    pub commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Loaded,
    Error,
    Missing,
}
