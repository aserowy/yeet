use std::path::Path;

use syntect::{
    easy::HighlightLines,
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use tokio::fs;

use crate::event::Preview;

pub async fn highlight(syntaxes: &SyntaxSet, theme: &Theme, path: &Path) -> Preview {
    match fs::read_to_string(path).await {
        Ok(content) => {
            let syntax = resolve_syntax(syntaxes, &content, path).await;
            if let Some(syntax) = syntax {
                tracing::debug!("syntax: {:?}", syntax.name);

                let mut highlighter = HighlightLines::new(syntax, theme);
                let mut result = vec![];
                for line in LinesWithEndings::from(&content) {
                    let highlighted = match highlighter.highlight_line(line, syntaxes) {
                        Ok(ranges) => &as_24_bit_terminal_escaped(&ranges[..], false),
                        Err(err) => {
                            tracing::error!("unable to highlight line: {:?}", err);
                            line
                        }
                    };
                    result.push(highlighted.to_string());
                }

                Preview::Content(path.to_path_buf(), result)
            } else {
                tracing::debug!("unable to resolve syntax for: {:?}", path);

                let content: Vec<_> = content.lines().map(|l| l.to_string()).collect();

                Preview::Content(path.to_path_buf(), content)
            }
        }
        Err(err) => {
            tracing::error!("reading file failed: {:?} {:?}", path, err);
            Preview::None(path.to_path_buf())
        }
    }
}

async fn resolve_syntax<'a>(
    syntaxes: &'a SyntaxSet,
    content: &str,
    path: &Path,
) -> Option<&'a SyntaxReference> {
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

    syntaxes.find_syntax_by_first_line(content)
}
