use std::path::{Path, PathBuf};

use yeet_buffer::model::{BufferLine, Sign, SignIdentifier};

use crate::{
    model::{
        mark::{Marks, MARK_SIGN_ID},
        qfix::{QuickFix, QFIX_SIGN_ID},
        Buffer, DirectoryBuffer,
    },
    theme::Theme,
};

pub fn set_sign_if_qfix(qfix: &QuickFix, bl: &mut BufferLine, path: &Path, theme: &Theme) {
    let is_marked = qfix.entries.iter().any(|p| p == path);
    if !is_marked {
        return;
    }

    set(bl, QFIX_SIGN_ID, theme);
}

pub fn set_sign_if_marked(marks: &Marks, bl: &mut BufferLine, path: &Path, theme: &Theme) {
    let is_marked = marks.entries.values().any(|p| p == path);
    if !is_marked {
        return;
    }

    set(bl, MARK_SIGN_ID, theme);
}

pub fn set(bl: &mut BufferLine, sign_id: SignIdentifier, theme: &Theme) {
    let is_signed = bl.signs.iter().any(|s| s.id == sign_id);
    if is_signed {
        return;
    }

    if let Some(sign) = generate_sign(sign_id, theme) {
        bl.signs.push(sign);
    }
}

pub fn set_sign_for_paths(
    buffers: Vec<&mut Buffer>,
    paths: Vec<PathBuf>,
    sign_id: SignIdentifier,
    theme: &Theme,
) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            _ => continue,
        };

        set_sign_for_paths_in_buffer(buffer, &paths, sign_id, theme);
    }
}

pub fn generate_sign(sign_id: SignIdentifier, theme: &Theme) -> Option<Sign> {
    match sign_id {
        QFIX_SIGN_ID => Some(Sign {
            id: QFIX_SIGN_ID,
            content: 'c',
            style: theme.sign_qfix_style(),
            priority: 0,
        }),
        MARK_SIGN_ID => Some(Sign {
            id: MARK_SIGN_ID,
            content: 'm',
            style: theme.sign_mark_style(),
            priority: 0,
        }),
        _ => None,
    }
}

pub fn unset_sign_on_all_buffers(buffers: Vec<&mut Buffer>, sign_id: SignIdentifier) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            _ => continue,
        };

        for line in &mut buffer.buffer.lines {
            unset(line, sign_id);
        }
    }
}

pub fn unset_sign_for_paths(
    buffers: Vec<&mut Buffer>,
    paths: Vec<PathBuf>,
    sign_id: SignIdentifier,
) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            _ => continue,
        };

        unset_sign_for_paths_in_buffer(buffer, &paths, sign_id);
    }
}

fn set_sign_for_paths_in_buffer(
    buffer: &mut DirectoryBuffer,
    paths: &[PathBuf],
    sign_id: SignIdentifier,
    theme: &Theme,
) {
    for path in paths {
        let parent = match path.parent() {
            Some(it) => it,
            None => return,
        };

        if buffer.path.as_path() != parent {
            continue;
        }

        let file_name = match path.file_name() {
            Some(it) => match it.to_str() {
                Some(it) => it,
                None => return,
            },
            None => return,
        };

        if let Some(line) = buffer
            .buffer
            .lines
            .iter_mut()
            .find(|bl| bl.content.to_stripped_string() == file_name)
        {
            set(line, sign_id, theme);
        }
    }
}

fn unset_sign_for_paths_in_buffer(
    buffer: &mut DirectoryBuffer,
    paths: &[PathBuf],
    sign_id: SignIdentifier,
) {
    for path in paths {
        let parent = match path.parent() {
            Some(it) => it,
            None => return,
        };

        if buffer.path.as_path() != parent {
            continue;
        }

        let file_name = match path.file_name() {
            Some(it) => match it.to_str() {
                Some(it) => it,
                None => return,
            },
            None => return,
        };

        if let Some(line) = buffer
            .buffer
            .lines
            .iter_mut()
            .find(|bl| bl.content.to_stripped_string() == file_name)
        {
            unset(line, sign_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::qfix::QFIX_SIGN_ID;
    use crate::model::mark::MARK_SIGN_ID;

    #[test]
    fn generate_sign_uses_theme_qfix_style() {
        let theme = Theme::default();
        let sign = generate_sign(QFIX_SIGN_ID, &theme).expect("qfix sign should be Some");
        assert_eq!(sign.style, theme.sign_qfix_style());
        assert_eq!(sign.content, 'c');
    }

    #[test]
    fn generate_sign_uses_theme_mark_style() {
        let theme = Theme::default();
        let sign = generate_sign(MARK_SIGN_ID, &theme).expect("mark sign should be Some");
        assert_eq!(sign.style, theme.sign_mark_style());
        assert_eq!(sign.content, 'm');
    }

    #[test]
    fn generate_sign_custom_theme_colors() {
        use ratatui::style::Color;
        let mut theme = Theme::default();
        theme.set_color(crate::theme::tokens::SIGN_QFIX.to_string(), Color::Rgb(255, 0, 0));

        let sign = generate_sign(QFIX_SIGN_ID, &theme).expect("qfix sign should be Some");
        assert!(sign.style.contains("\x1b[38;2;255;0;0m"), "sign style should contain custom red color");
    }
}

pub fn unset(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let position = bl.signs.iter().position(|s| s.id == sign_id);
    if let Some(position) = position {
        bl.signs.remove(position);
    }
}
