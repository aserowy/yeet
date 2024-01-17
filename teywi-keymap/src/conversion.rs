use crossterm::event::{self, KeyEvent, KeyEventKind, ModifierKeyCode};

use crate::{KeyCode, KeyPress, KeyModifier};

pub fn to_keypress(event: KeyEvent) -> Option<KeyPress> {
    match event.code {
        // event::KeyCode::Backspace => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Enter => resolve_keypress_for_key(event.kind, KeyCode::Enter),
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
        event::KeyCode::Char(c) => resolve_keypress_for_key(event.kind, KeyCode::from_char(c)),
        // event::KeyCode::Null => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Esc => resolve_keypress_for_key(event.kind, KeyCode::Esc),
        // event::KeyCode::CapsLock => todo!(),
        // event::KeyCode::ScrollLock => None,
        // event::KeyCode::NumLock => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::PrintScreen => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Pause => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::Menu => resolve_keypress_for_key(event.kind, KeyCode::),
        // event::KeyCode::KeypadBegin => None,
        // event::KeyCode::Media(_) => resolve_keypress_for_key(event.kind, KeyCode::),
        event::KeyCode::Modifier(m) => resolve_keypress_for_mod(event.kind, m),
        _ => None,
    }
}

fn resolve_keypress_for_key(kind: KeyEventKind, code: KeyCode) -> Option<KeyPress> {
    if kind != KeyEventKind::Press {
        return None;
    }

    Some(KeyPress::Key(code))
}

fn resolve_keypress_for_mod(kind: KeyEventKind, modifier: ModifierKeyCode) -> Option<KeyPress> {
    let active = kind == KeyEventKind::Press;

    match modifier {
        ModifierKeyCode::LeftShift => Some(KeyPress::Modifier(KeyModifier::Shift, active)),
        ModifierKeyCode::LeftControl => Some(KeyPress::Modifier(KeyModifier::Ctrl, active)),
        ModifierKeyCode::LeftAlt => Some(KeyPress::Modifier(KeyModifier::Alt, active)),
        ModifierKeyCode::LeftSuper => Some(KeyPress::Modifier(KeyModifier::Command, active)),
        ModifierKeyCode::LeftHyper => Some(KeyPress::Modifier(KeyModifier::Command, active)),
        ModifierKeyCode::LeftMeta => Some(KeyPress::Modifier(KeyModifier::Alt, active)),
        ModifierKeyCode::RightShift => Some(KeyPress::Modifier(KeyModifier::Shift, active)),
        ModifierKeyCode::RightControl => Some(KeyPress::Modifier(KeyModifier::Ctrl, active)),
        ModifierKeyCode::RightAlt => Some(KeyPress::Modifier(KeyModifier::Alt, active)),
        ModifierKeyCode::RightSuper => Some(KeyPress::Modifier(KeyModifier::Command, active)),
        ModifierKeyCode::RightHyper => Some(KeyPress::Modifier(KeyModifier::Command, active)),
        ModifierKeyCode::RightMeta => Some(KeyPress::Modifier(KeyModifier::Alt, active)),
        ModifierKeyCode::IsoLevel3Shift => Some(KeyPress::Modifier(KeyModifier::Shift, active)),
        ModifierKeyCode::IsoLevel5Shift => Some(KeyPress::Modifier(KeyModifier::Shift, active)),
    }
}
