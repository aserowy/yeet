use std::path::Path;

use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use tokio::{
    fs::{self, File},
    io::{AsyncBufReadExt, BufReader},
};

pub async fn highlight(path: &Path) -> Option<String> {
    match fs::read_to_string(path).await {
        Ok(content) => {
            // TODO: load once and cache
            let ts = ThemeSet::load_defaults();
            let ps = SyntaxSet::load_defaults_newlines();
            let syntax = resolve_syntax(&ps, path).await;
            if let Some(syntax) = syntax {
                tracing::debug!("syntax: {:?}", syntax.name);
                let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

                let mut sb = String::new();
                for line in LinesWithEndings::from(&content) {
                    let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                    sb.push_str(&escaped);
                }

                Some(sb)
            } else {
                None
            }
        }
        Err(err) => {
            tracing::error!("reading file failed: {:?} {:?}", path, err);
            None
        }
    }
}

async fn resolve_syntax<'a>(syntaxes: &'a SyntaxSet, path: &Path) -> Option<&'a SyntaxReference> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let syntax = syntaxes.find_syntax_by_extension(&name);
    if syntax.is_some() {
        return syntax;
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy())
        .unwrap_or_default();
    let syntax = syntaxes.find_syntax_by_extension(&ext);
    if syntax.is_some() {
        return syntax;
    }

    if let Ok(file) = File::open(&path).await {
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        let result = reader.read_line(&mut line).await;

        if result.is_ok() {
            syntaxes.find_syntax_by_first_line(&line)
        } else {
            tracing::error!("reading first line failed: {:?}", result.err());
            None
        }
    } else {
        None
    }
}
