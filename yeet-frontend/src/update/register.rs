use yeet_buffer::{
    message::{BufferMessage, CursorDirection},
    model::Mode,
};
use yeet_keymap::message::{Envelope, KeySequence, Message};

use crate::model::register::{Register, RegisterScope};

// TODO: handle search scope as well
#[tracing::instrument(skip(mode, register, envelope))]
pub fn start_scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    // TODO: @@
    if mode.is_command() {
        return;
    }

    if let Some(scope) = resolve_register_scope(&envelope.messages) {
        tracing::trace!("starting scope: {:?}", scope);

        register
            .scopes
            .entry(scope)
            .or_insert_with(|| "".to_owned());
    }
}

fn resolve_register_scope(messages: &[Message]) -> Option<RegisterScope> {
    if is_dot_scope(messages) {
        Some(RegisterScope::Dot)
    } else if is_find_scope(messages) {
        Some(RegisterScope::Find)
    } else {
        resolve_macro_register(messages).map(RegisterScope::Macro)
    }
}

fn is_dot_scope(messages: &[Message]) -> bool {
    messages.iter().any(|message| {
        matches!(
            message,
            Message::Buffer(BufferMessage::ChangeMode(_, Mode::Insert))
                | Message::Buffer(BufferMessage::Modification(..))
        )
    })
}

fn is_find_scope(messages: &[Message]) -> bool {
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

#[tracing::instrument(skip(mode, register, envelope))]
pub fn finish_scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    let sequence = match &envelope.sequence {
        KeySequence::Completed(sequence) => sequence.as_str(),
        KeySequence::Changed(_) | KeySequence::None => return,
    };

    let is_macro_start = resolve_macro_register(&envelope.messages).is_some();
    let is_macro_stop = finish_macro_scope(&envelope.messages);

    let mut to_close = Vec::new();
    for (scope, content) in register.scopes.iter_mut() {
        match scope {
            RegisterScope::Dot | RegisterScope::Find => {
                if finish_mode_dependend_scope(mode, scope) {
                    to_close.push(scope.clone());
                }
                content.push_str(sequence);
            }
            RegisterScope::Macro(_) => {
                if is_macro_start {
                    continue;
                } else if is_macro_stop {
                    to_close.push(scope.clone());
                    continue;
                }
                content.push_str(sequence);
            }
        }
    }

    for scope in to_close {
        tracing::trace!("closing scope: {:?}", scope);

        if let Some(content) = register.scopes.remove(&scope) {
            match scope {
                RegisterScope::Dot => register.dot.replace(content),
                RegisterScope::Find => register.find.replace(content),
                RegisterScope::Macro(identifier) => register.content.insert(identifier, content),
            };
        }
    }
}

fn finish_mode_dependend_scope(mode: &Mode, scope: &RegisterScope) -> bool {
    mode != &Mode::Insert && !matches!(scope, RegisterScope::Macro(_))
}

fn finish_macro_scope(messages: &[Message]) -> bool {
    messages.iter().any(|m| m == &Message::StopMacro)
}
