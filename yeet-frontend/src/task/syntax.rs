use std::path::Path;

use syntect::{
    easy::HighlightLines,
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use tokio::fs;

use super::sanitize::strip_non_sgr_escape_sequences;
use crate::event::Preview;

pub async fn highlight(
    syntaxes: &SyntaxSet,
    theme: &Theme,
    path: &Path,
    content_width: u16,
) -> Preview {
    match fs::read_to_string(path).await {
        Ok(content) => {
            let syntax = resolve_syntax(syntaxes, &content, path).await;
            if let Some(syntax) = syntax {
                tracing::debug!("syntax: {:?}", syntax.name);

                let max_chars = content_width as usize;
                let mut highlighter = HighlightLines::new(syntax, theme);
                let mut result = vec![];
                for line in LinesWithEndings::from(&content) {
                    let truncated = truncate_line(line, max_chars);
                    let highlighted = match highlighter.highlight_line(&truncated, syntaxes) {
                        Ok(ranges) => &as_24_bit_terminal_escaped(&ranges[..], false),
                        Err(err) => {
                            tracing::error!("unable to highlight line: {:?}", err);
                            &truncated
                        }
                    };
                    result.push(strip_non_sgr_escape_sequences(highlighted));
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

fn truncate_line(line: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return line.to_string();
    }

    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
    if trimmed.chars().count() <= max_chars {
        return line.to_string();
    }

    trimmed.chars().take(max_chars).collect::<String>() + "\n"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_line_unchanged() {
        assert_eq!(truncate_line("hello\n", 10), "hello\n");
    }

    #[test]
    fn truncate_long_line() {
        assert_eq!(truncate_line("hello world foo\n", 5), "hello\n");
    }

    #[test]
    fn truncate_exact_length() {
        assert_eq!(truncate_line("hello\n", 5), "hello\n");
    }

    #[test]
    fn truncate_zero_width_passes_through() {
        assert_eq!(truncate_line("hello\n", 0), "hello\n");
    }

    #[test]
    fn truncate_line_without_newline() {
        assert_eq!(truncate_line("hello world", 5), "hello\n");
    }

    #[test]
    fn truncate_empty_line() {
        assert_eq!(truncate_line("\n", 10), "\n");
    }
}
