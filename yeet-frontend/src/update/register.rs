use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{Envelope, KeySequence, Message};

use crate::model::register::{Register, RegisterScope};

pub fn scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    if register.scope.is_some() {
        return;
    }

    if !is_command_mode(mode) && contains_valid_dot(&envelope.messages) {
        register.scope = Some(RegisterScope::Dot);
        register.dot = None;
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

pub fn finish(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    let sequence = match &envelope.sequence {
        KeySequence::Completed(sequence) => sequence,
        KeySequence::Changed(_) | KeySequence::None => return,
    };

    let scope = match &register.scope {
        Some(scope) => scope.clone(),
        None => return,
    };

    match scope {
        RegisterScope::_Command => todo!(),
        RegisterScope::Dot => {
            if let Some(dot) = &mut register.dot {
                dot.push_str(&sequence.to_string());
            } else {
                register.dot = Some(sequence.to_string());
            }
        }
        RegisterScope::_Find => todo!(),
        RegisterScope::_Macro(_) => todo!(),
        RegisterScope::_Search => todo!(),
    };

    // TODO: extend for macros
    if mode != &Mode::Insert {
        register.scope = None;
    }
}
