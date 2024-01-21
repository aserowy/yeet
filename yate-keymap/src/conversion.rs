use crossterm::event::{self, KeyEvent, KeyEventKind};

use crate::key::{Key, KeyCode, KeyModifier};

pub fn to_key(event: &KeyEvent) -> Option<Key> {
    let modifier = event
        .modifiers
        .iter_names()
        .map(|(s, _)| to_modifier(s))
        .filter(|m| m.is_some())
        .flatten()
        .collect();

    match event.code {
        // event::KeyCode::Backspace => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Enter => resolve_keypress_for_key(event.kind, KeyCode::Enter, modifier),
        // event::KeyCode::Left => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Right => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Up => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Down => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Home => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::End => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::PageUp => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::PageDown => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Tab => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::BackTab => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Delete => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Insert => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::F(_) => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Char(c) => {
            resolve_keypress_for_key(event.kind, KeyCode::from_char(c), modifier)
        }
        // event::KeyCode::Null => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Esc => resolve_keypress_for_key(event.kind, KeyCode::Esc, modifier),
        // event::KeyCode::CapsLock => todo!(),
        // event::KeyCode::ScrollLock => None,
        // event::KeyCode::NumLock => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::PrintScreen => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Pause => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Menu => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::KeypadBegin => None,
        // event::KeyCode::Media(_) => resolve_keypress_for_key(event.kind, KeyCode::),
        _ => None,
    }
}

fn resolve_keypress_for_key(
    kind: KeyEventKind,
    code: KeyCode,
    modifier: Vec<KeyModifier>,
) -> Option<Key> {
    if kind != KeyEventKind::Press {
        return None;
    }

    Some(Key::new(code, modifier))
}

fn to_modifier(modifier: &str) -> Option<KeyModifier> {
    match modifier {
        "ALT" => Some(KeyModifier::Alt),
        "CONTROL" => Some(KeyModifier::Ctrl),
        "HYPER" => Some(KeyModifier::Command),
        "META" => Some(KeyModifier::Alt),
        "SHIFT" => Some(KeyModifier::Shift),
        "SUPER" => Some(KeyModifier::Command),
        _ => None,
    }
}
