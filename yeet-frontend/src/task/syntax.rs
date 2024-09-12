use std::path::Path;

use syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use tokio::{
    fs::{self, File},
    io::{AsyncBufReadExt, BufReader},
};

pub async fn highlight(syntaxes: &SyntaxSet, theme: &Theme, path: &Path) -> Option<String> {
    match fs::read_to_string(path).await {
        Ok(content) => {
            let syntax = resolve_syntax(syntaxes, path).await;
            if let Some(syntax) = syntax {
                tracing::debug!("syntax: {:?}", syntax.name);

                let mut highlighter = HighlightLines::new(syntax, theme);
                let mut result = String::new();
                for line in LinesWithEndings::from(&content) {
                    let ranges: Vec<(Style, &str)> =
                        highlighter.highlight_line(line, &syntaxes).unwrap();
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                    result.push_str(&escaped);
                }

                Some(result)
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
