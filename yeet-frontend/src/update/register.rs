use yeet_buffer::{
    message::{BufferMessage, CursorDirection},
    model::Mode,
};
use yeet_keymap::message::{Envelope, KeySequence, Message, MessageSource};

use crate::model::register::{Register, RegisterScope};

#[tracing::instrument(skip(mode, register, envelope))]
pub fn scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    if register.scope.is_some()
        || envelope.source != MessageSource::User
        || matches!(mode, Mode::Command(_))
    {
        return;
    }

    if start_dot_scope(&envelope.messages) {
        register.scope = Some(RegisterScope::Dot);
        register.dot = None;
    } else if start_find_scope(&envelope.messages) {
        register.scope = Some(RegisterScope::Find);
        register.find = None;
    } else if let Some(identifier) = resolve_macro_register(&envelope.messages) {
        register.scope = Some(RegisterScope::Macro(identifier));
    }

    if register.scope.is_some() {
        tracing::trace!("starting scope: {:?}", register.scope);
    }
}

fn start_dot_scope(messages: &[Message]) -> bool {
    let starts_insert = messages.iter().any(|m| {
        matches!(
            m,
            Message::Buffer(BufferMessage::ChangeMode(_, Mode::Insert))
        )
    });

    let is_modification = messages
        .iter()
        .any(|m| matches!(m, Message::Buffer(BufferMessage::Modification(..))));

    starts_insert || is_modification
}

fn start_find_scope(messages: &[Message]) -> bool {
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

#[tracing::instrument(skip(mode, register, envelope))]
pub fn finish(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    let sequence = match &envelope.sequence {
        KeySequence::Completed(sequence) => sequence.as_str(),
        KeySequence::Changed(_) | KeySequence::None => return,
    };

    let scope = match &register.scope {
        Some(scope) => scope.clone(),
        None => return,
    };

    let is_macro_start = resolve_macro_register(&envelope.messages).is_some();
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
        RegisterScope::Macro(identifier) => {
            tracing::trace!("writing to macro at register {:?}", identifier);

            if is_macro_start {
                register.content.remove(&identifier);
            } else if let Some(content) = register.content.get_mut(&identifier) {
                content.push_str(sequence);
            } else {
                register.content.insert(identifier, sequence.to_string());
            }
        }
    };

    if finish_mode_dependend_scope(mode, &scope) || finish_macro_scope(&envelope.messages) {
        tracing::trace!("closing scope: {:?}", scope);

        register.scope = None;
    }
}

fn finish_mode_dependend_scope(mode: &Mode, scope: &RegisterScope) -> bool {
    mode != &Mode::Insert && !matches!(scope, RegisterScope::Macro(_))
}

fn finish_macro_scope(messages: &[Message]) -> bool {
    messages.iter().any(|m| m == &Message::StopMacro)
}

fn resolve_macro_register(messages: &[Message]) -> Option<char> {
    let message = messages
        .iter()
        .find(|m| matches!(m, Message::StartMacro(_)));

    if let Some(Message::StartMacro(identifier)) = message {
        Some(*identifier)
    } else {
        None
    }
}
