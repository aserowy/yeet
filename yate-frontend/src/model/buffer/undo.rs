use std::time;

use yate_keymap::message::Mode;

#[derive(Debug, Default)]
pub struct Undo {
    current_change: Option<BufferChanged>,
    current_save_index: Option<usize>,
    current_transaction_index: Option<usize>,
    change_buffer: Vec<BufferChanged>,
    transactions: Vec<Transaction>,
}

impl Undo {
    pub fn add(&mut self, mode: &Mode, changes: Vec<BufferChanged>) {
        if mode == &Mode::Insert {
            for change in changes {
                let (pushed, updated) = update(&self.current_change, &change);
                if let Some(pushed) = pushed {
                    self.change_buffer.push(pushed);
                }

                self.current_change = Some(updated);
            }
        } else {
            self.add_transaction(changes);
        }
    }

    pub fn close_transaction(&mut self) {
        if let Some(current) = &self.current_change {
            self.change_buffer.push(current.clone());
            self.current_change = None;
        }

        if self.change_buffer.is_empty() {
            return;
        }

        let changes = self.change_buffer.clone();
        self.add_transaction(changes);
        self.change_buffer = Vec::new();
    }

    pub fn save(&mut self) -> Vec<BufferChanged> {
        self.close_transaction();

        if self.transactions.is_empty() {
            return Vec::new();
        }

        let start = if let Some(index) = self.current_save_index {
            index
        } else {
            0
        };

        let end = if let Some(index) = self.current_transaction_index {
            index + 1
        } else {
            self.current_transaction_index = Some(self.transactions.len() - 1);
            self.transactions.len()
        };

        let changes = self.transactions[start..end]
            .iter()
            .fold(Vec::new(), |mut acc, t| {
                acc.extend(t._changes.clone());
                acc
            });

        self.current_save_index = Some(self.transactions.len() - 1);

        changes
    }

    fn add_transaction(&mut self, changes: Vec<BufferChanged>) {
        if changes.is_empty() {
            return;
        }

        let timestamp = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(time) => time.as_secs(),
            Err(_) => 0,
        };

        self.current_transaction_index = Some(self.transactions.len());
        self.transactions.push(Transaction {
            _changes: changes,
            _timestamp: timestamp,
        });
    }
}

#[derive(Debug)]
struct Transaction {
    _changes: Vec<BufferChanged>,
    _timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BufferChanged {
    Content(usize, String, String),
    LineAdded(usize, String),
    LineRemoved(usize, String),
}

fn update(
    current: &Option<BufferChanged>,
    new: &BufferChanged,
) -> (Option<BufferChanged>, BufferChanged) {
    match new {
        BufferChanged::Content(ln, _, cntnt) => {
            if let Some(current) = current {
                let updated = update_current(current, ln, cntnt);
                if let Some(updated) = updated {
                    (None, updated)
                } else {
                    (Some(current.clone()), new.clone())
                }
            } else {
                (None, new.clone())
            }
        }
        BufferChanged::LineAdded(_, _) => (current.clone(), new.clone()),
        BufferChanged::LineRemoved(_, _) => (current.clone(), new.clone()),
    }
}

fn update_current(
    current: &BufferChanged,
    new_line_number: &usize,
    new_content: &str,
) -> Option<BufferChanged> {
    match current {
        BufferChanged::Content(current_ln, ld, _) => {
            if current_ln == new_line_number {
                Some(BufferChanged::Content(
                    *current_ln,
                    ld.to_string(),
                    new_content.to_string(),
                ))
            } else {
                None
            }
        }
        BufferChanged::LineAdded(current_ln, _) => {
            if current_ln == new_line_number {
                Some(BufferChanged::LineAdded(
                    *current_ln,
                    new_content.to_string(),
                ))
            } else {
                None
            }
        }
        BufferChanged::LineRemoved(_, _) => None,
    }
}
