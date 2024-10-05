use std::collections::HashSet;

use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::Action,
    event::Message,
    model::{
        junkyard::{FileEntryStatus, FileTransaction, JunkYard},
        mark::Marks,
        qfix::QuickFix,
        register::Register,
    },
    update::junkyard::get_junkyard_transaction,
};

pub fn print_marks(marks: &Marks) -> Vec<PrintContent> {
    let mut marks: Vec<_> = marks
        .entries
        .iter()
        .map(|(key, path)| (key, path.to_string_lossy().to_string()))
        .map(|(key, path)| format!("{:<4} {}", key, path))
        .collect();

    marks.sort();

    let mut contents = vec![":marks".to_string(), "Char Content".to_string()];
    contents.extend(marks);

    contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect()
}

pub fn tasks(tasks: &HashSet<String>) -> Vec<PrintContent> {
    let tasks = tasks.iter().map(|id| id.as_str());
    let mut contents = vec![":tasks", "Id"];
    contents.extend(tasks);

    contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect()
}

pub fn print_qfix_list(qfix: &QuickFix) -> Vec<Action> {
    let max_width = (qfix.entries.len() + 1).to_string().len();

    let entries: Vec<_> = qfix
        .entries
        .iter()
        .enumerate()
        .map(|(i, path)| (i + 1, path.to_string_lossy().to_string()))
        .map(|(i, path)| format!("{:>max_width$} {}", i, path))
        .collect();

    let mut contents = vec![":cl".to_string()];
    if entries.is_empty() {
        contents.push("no entries".to_string());
    } else {
        contents.extend(entries);
    }

    let content = contents
        .iter()
        .enumerate()
        .map(|(i, cntnt)| {
            if i == qfix.current_index + 1 {
                PrintContent::Information(cntnt.to_string())
            } else {
                PrintContent::Default(cntnt.to_string())
            }
        })
        .collect();

    vec![Action::EmitMessages(vec![Message::Keymap(
        KeymapMessage::Print(content),
    )])]
}

pub fn print_junkyard(junkyard: &JunkYard) -> Vec<PrintContent> {
    let mut contents = vec![":junk".to_string(), "Name Content".to_string()];
    if let Some(current) = get_junkyard_transaction(junkyard, &'"') {
        contents.push(print_junkyard_entry("\"\"", current));
    }
    if let Some(yanked) = &junkyard.yanked {
        contents.push(print_junkyard_entry("\"0", yanked));
    }
    for (index, entry) in junkyard.trashed.iter().enumerate() {
        let junk_name = format!("\"{}", index + 1);
        contents.push(print_junkyard_entry(&junk_name, entry));
    }

    contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect()
}

fn print_junkyard_entry(junk: &str, transaction: &FileTransaction) -> String {
    let is_ready = transaction
        .entries
        .iter()
        .all(|entry| entry.status == FileEntryStatus::Ready);

    let content = if is_ready {
        transaction
            .entries
            .iter()
            .map(|entry| entry.target.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "Processing".to_string()
    };

    format!("{:<4} {}", junk, content)
}

pub fn print_register(register: &Register) -> Vec<PrintContent> {
    let mut contents = vec![":reg".to_string(), "Name Content".to_string()];

    for (key, content) in register.content.iter() {
        contents.push(print_content(key, content));
    }

    if let Some(last_macro) = &register.last_macro {
        contents.push(print_content(&'@', last_macro));
    }
    if let Some(dot) = &register.dot {
        contents.push(print_content(&'.', dot));
    }
    if let Some(command) = &register.command {
        contents.push(print_content(&':', command));
    }
    if let Some(searched) = &register.searched {
        contents.push(print_content(&'/', &searched.1));
    }

    contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect()
}

fn print_content(prefix: &char, content: &str) -> String {
    format!("\"{:<3} {}", prefix, content)
}
