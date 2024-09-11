use std::path::Path;

use tokio::fs;

pub async fn highlight(path: &Path) -> Option<String> {
    match fs::read_to_string(path).await {
        Ok(content) => {
            Some(content)
        },
        Err(err) => {
            tracing::error!("reading file failed: {:?} {:?}", path, err);
            None
        }
    }
}
