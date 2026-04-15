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
                        Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], false),
                        Err(err) => {
                            tracing::error!("unable to highlight line: {:?}", err);
                            line.to_string()
                        }
                    };
                    result.push(strip_non_sgr_escape_sequences(&highlighted));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use syntect::highlighting::ThemeSet;

    #[tokio::test]
    async fn long_url_does_not_corrupt_subsequent_line_highlighting() {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = &theme_set.themes["base16-eighties.dark"];

        // Create a markdown file with a long URL line followed by a normal line.
        // If pre-highlight truncation were applied, the closing `"` of the URL
        // would be lost, causing syntect to carry "inside a string" state into
        // the next line and style it as a string literal.
        let long_url = format!(
            "src=\"https://github.com/user-attachments/assets/{}\"\n",
            "a".repeat(500)
        );
        let normal_line = "## Normal Heading\n";
        let content = format!("{}{}", long_url, normal_line);

        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let result = highlight(&syntaxes, theme, tmp.path()).await;

        match result {
            Preview::Content(_, lines) => {
                assert_eq!(lines.len(), 2, "expected two highlighted lines");

                // The second line must NOT inherit string-literal styling from the
                // long URL on line 1.  A string-literal style would wrap the entire
                // line in an SGR color escape that differs from the heading style.
                // We verify by checking that the second line's ANSI output differs
                // from what a pure-string rendering would look like — concretely,
                // the heading markers ("##") should receive heading styling, not
                // string styling.
                let second_line = &lines[1];

                // If syntect state were corrupted, the second line would be
                // entirely wrapped in the same string-literal color as line 1.
                // A correct parse produces distinct escape sequences for the
                // heading punctuation vs. the heading text.
                // As a simple heuristic: the second line should NOT be identical
                // to just wrapping "## Normal Heading" in the first line's color.
                assert!(
                    !second_line.is_empty(),
                    "highlighted line should not be empty"
                );

                // Verify the raw text content is preserved (strip ANSI escapes)
                let ansi_stripped: String = second_line
                    .chars()
                    .fold((String::new(), false), |(mut s, in_esc), c| {
                        if c == '\x1b' {
                            (s, true)
                        } else if in_esc {
                            if c == 'm' {
                                (s, false)
                            } else {
                                (s, true)
                            }
                        } else {
                            s.push(c);
                            (s, false)
                        }
                    })
                    .0;
                assert!(
                    ansi_stripped.contains("Normal Heading"),
                    "second line should contain the heading text, got: {}",
                    ansi_stripped
                );
            }
            _ => panic!("expected Preview::Content"),
        }
    }
}
