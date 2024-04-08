use yeet_buffer::{
    message::{BufferMessage, CursorDirection},
    model::Mode,
};
use yeet_keymap::message::{Envelope, KeySequence, Message, MessageSource};

use crate::model::register::{Register, RegisterScope};

pub fn scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    if register.scope.is_some() || envelope.source != MessageSource::User || is_command_mode(mode) {
        return;
    }

    if contains_valid_dot(&envelope.messages) {
        register.scope = Some(RegisterScope::Dot);
        register.dot = None;
    } else if contains_valid_find(&envelope.messages) {
        register.scope = Some(RegisterScope::Find);
        register.find = None;
    }
}

fn is_command_mode(mode: &Mode) -> bool {
    matches!(mode, Mode::Command(_))
}

fn contains_valid_dot(messages: &[Message]) -> bool {
    messages
        .iter()
        .any(|m| matches!(m, Message::Buffer(BufferMessage::Modification(..))))
}

fn contains_valid_find(messages: &[Message]) -> bool {
    let direction = messages.iter().find_map(|m| {
        if let Message::Buffer(BufferMessage::MoveCursor(_, direction)) = m {
            Some(direction)
        } else {
            None
        }
    });

    if let Some(direction) = direction {
        match direction {
            CursorDirection::FindBackward(_)
            | CursorDirection::FindForward(_)
            | CursorDirection::TillBackward(_)
            | CursorDirection::TillForward(_) => true,

            CursorDirection::Bottom
            | CursorDirection::Down
            | CursorDirection::Left
            | CursorDirection::LineEnd
            | CursorDirection::LineStart
            | CursorDirection::Right
            | CursorDirection::Search(_)
            | CursorDirection::Top
            | CursorDirection::Up => false,
        }
    } else {
        false
    }
}

pub fn finish(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    let sequence = match &envelope.sequence {
        KeySequence::Completed(sequence) => sequence.as_str(),
        KeySequence::Changed(_) | KeySequence::None => return,
    };

    let scope = match &register.scope {
        Some(scope) => scope.clone(),
        None => return,
    };

    match scope {
        RegisterScope::Dot => {
            if let Some(dot) = &mut register.dot {
                dot.push_str(sequence);
            } else {
                register.dot = Some(sequence.to_string());
            }
        }
        RegisterScope::Find => {
            if let Some(find) = &mut register.find {
                find.push_str(sequence);
            } else {
                register.find = Some(sequence.to_string());
            }
        }
        RegisterScope::_Macro(_) => todo!(),
    };

    // TODO: extend for macros
    if mode != &Mode::Insert {
        register.scope = None;
    }
}
