use yeet_buffer::{
    message::BufferMessage,
    model::{Mode, SearchDirection},
};
use yeet_keymap::message::{Envelope, KeySequence, Message};

use crate::{
    action::Action,
    model::register::{Register, RegisterScope},
};

pub fn get_register(register: &Register, register_id: &char) -> Option<String> {
    match register_id {
        '@' => register.last_macro.clone(),
        '.' => register.dot.clone(),
        ':' => register.command.clone(),
        '/' => register.searched.as_ref().map(|sd| sd.1.clone()),
        char => register.content.get(char).cloned(),
    }
}

pub fn replay_register(register: &mut Register, char: &char) -> Vec<Action> {
    if let Some(content) = get_register(register, char) {
        vec![Action::EmitMessages(vec![Message::ExecuteKeySequence(
            content.to_string(),
        )])]
    } else {
        Vec::new()
    }
}

pub fn replay_macro_register(register: &mut Register, char: &char) -> Vec<Action> {
    if let Some(content) = get_register(register, char) {
        register.last_macro = Some(content.to_string());
        vec![Action::EmitMessages(vec![Message::ExecuteKeySequence(
            content.to_string(),
        )])]
    } else {
        Vec::new()
    }
}

pub fn get_direction_from_search_register(register: &Register) -> Option<&SearchDirection> {
    register.searched.as_ref().map(|sd| &sd.0)
}

pub fn get_macro_register(register: &Register) -> Option<&RegisterScope> {
    register
        .scopes
        .keys()
        .find(|scope| matches!(scope, RegisterScope::Macro(_)))
}

#[tracing::instrument(skip(mode, register, envelope))]
pub fn start_register_scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    if let Some(scope) = resolve_register_scope(mode, &envelope.messages) {
        tracing::trace!("starting scope: {:?}", scope);

        register
            .scopes
            .entry(scope)
            .or_insert_with(|| "".to_owned());
    }
}

fn resolve_register_scope(mode: &Mode, messages: &[Message]) -> Option<RegisterScope> {
    if is_dot_scope(mode, messages) {
        Some(RegisterScope::Dot)
    } else {
        resolve_macro_register(messages).map(RegisterScope::Macro)
    }
}

fn is_dot_scope(mode: &Mode, messages: &[Message]) -> bool {
    if mode.is_command() {
        return false;
    }

    messages.iter().any(|message| {
        matches!(
            message,
            Message::Buffer(BufferMessage::ChangeMode(_, Mode::Insert))
                | Message::Buffer(BufferMessage::Modification(..))
        )
    })
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
pub fn finish_register_scope(mode: &Mode, register: &mut Register, envelope: &Envelope) {
    let sequence = match &envelope.sequence {
        KeySequence::Completed(sequence) => sequence.as_str(),
        KeySequence::Changed(_) | KeySequence::None => return,
    };

    let mut to_close = Vec::new();
    for (scope, content) in register.scopes.iter_mut() {
        match scope {
            RegisterScope::Dot => {
                if mode != &Mode::Insert {
                    to_close.push(scope.clone());
                }

                content.push_str(sequence);
            }
            RegisterScope::Macro(_) => {
                let is_macro_start = resolve_macro_register(&envelope.messages).is_some();
                if is_macro_start {
                    continue;
                } else if envelope.messages.iter().any(|m| m == &Message::StopMacro) {
                    to_close.push(scope.clone());
                } else {
                    content.push_str(sequence);
                }
            }
        }
    }

    for scope in to_close {
        tracing::trace!("closing scope: {:?}", scope);

        if let Some(content) = register.scopes.remove(&scope) {
            match scope {
                RegisterScope::Dot => register.dot.replace(content),
                RegisterScope::Macro(identifier) => register.content.insert(identifier, content),
            };
        }
    }
}
