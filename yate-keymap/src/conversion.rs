use crossterm::event::{self, KeyEvent, KeyEventKind};

use crate::key::{Key, KeyCode, KeyModifier};

pub fn to_key(event: &KeyEvent) -> Option<Key> {
    let modifier = event
        .modifiers
        .iter_names()
        .flat_map(|(s, _)| to_modifier(s))
        .collect();

    match event.code {
        event::KeyCode::Backspace => resolve(event.kind, KeyCode::Backspace, modifier),
        event::KeyCode::Enter => resolve(event.kind, KeyCode::Enter, modifier),
        event::KeyCode::Left => resolve(event.kind, KeyCode::Left, modifier),
        event::KeyCode::Right => resolve(event.kind, KeyCode::Right, modifier),
        event::KeyCode::Up => resolve(event.kind, KeyCode::Up, modifier),
        event::KeyCode::Down => resolve(event.kind, KeyCode::Down, modifier),
        // event::KeyCode::Home => resolve(event.kind, KeyCode::),
        // event::KeyCode::End => resolve(event.kind, KeyCode::),
        // event::KeyCode::PageUp => resolve(event.kind, KeyCode::),
        // event::KeyCode::PageDown => resolve(event.kind, KeyCode::),
        event::KeyCode::Tab => resolve(event.kind, KeyCode::Tab, modifier),
        // event::KeyCode::BackTab => resolve(event.kind, KeyCode::),
        event::KeyCode::Delete => resolve(event.kind, KeyCode::Delete, modifier),
        // event::KeyCode::Insert => resolve(event.kind, KeyCode::),
        // event::KeyCode::F(_) => resolve(event.kind, KeyCode::),
        event::KeyCode::Char(c) => resolve(event.kind, KeyCode::from_char(c), modifier),
        // event::KeyCode::Null => resolve(event.kind, KeyCode::),
        event::KeyCode::Esc => resolve(event.kind, KeyCode::Esc, modifier),
        // event::KeyCode::CapsLock => todo!(),
        // event::KeyCode::ScrollLock => None,
        // event::KeyCode::NumLock => resolve(event.kind, KeyCode::),
        // event::KeyCode::PrintScreen => resolve(event.kind, KeyCode::),
        // event::KeyCode::Pause => resolve(event.kind, KeyCode::),
        // event::KeyCode::Menu => resolve(event.kind, KeyCode::),
        // event::KeyCode::KeypadBegin => None,
        // event::KeyCode::Media(_) => resolve(event.kind, KeyCode::),
        _ => None,
    }
}

fn resolve(kind: KeyEventKind, code: KeyCode, modifier: Vec<KeyModifier>) -> Option<Key> {
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
