use std::time;

use super::Mode;

#[derive(Debug)]
struct Transaction {
    changes: Vec<BufferChanged>,
    _timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BufferChanged {
    Content(usize, String, String),
    LineAdded(usize, String),
    LineRemoved(usize, String),
}

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

    pub fn get_uncommited_changes(&self) -> Vec<BufferChanged> {
        if self.transactions.is_empty() {
            return Vec::new();
        }

        let start = if let Some(index) = self.current_save_index {
            index + 1
        } else {
            0
        };

        let end = if let Some(index) = self.current_transaction_index {
            index
        } else {
            self.transactions.len() - 1
        };

        self.transactions[start..end + 1]
            .iter()
            .fold(Vec::new(), |mut acc, t| {
                acc.extend(t.changes.clone());
                acc
            })
    }

    pub fn save(&mut self) -> Vec<BufferChanged> {
        self.close_transaction();

        if self.transactions.is_empty() {
            return Vec::new();
        }

        let changes = self.get_uncommited_changes();

        self.current_save_index = if let Some(index) = self.current_transaction_index {
            Some(index)
        } else {
            Some(self.transactions.len() - 1)
        };

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

        self.transactions.push(Transaction {
            changes,
            _timestamp: timestamp,
        });

        self.current_transaction_index = Some(self.transactions.len() - 1);
    }
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

pub fn consolidate_modifications(changes: &Vec<BufferChanged>) -> Vec<BufferChanged> {
    let mut consolidated_changes = Vec::new();
    'changes: for change in changes {
        match change {
            BufferChanged::Content(line_index, _, new) => {
                let mut index = *line_index;
                for (rev_index, consolidated) in consolidated_changes.iter().rev().enumerate() {
                    match consolidated {
                        BufferChanged::Content(c_i, old, _) => {
                            if c_i == &index {
                                let max_index = consolidated_changes.len() - 1;
                                let corrected_vec_index = max_index - rev_index;

                                consolidated_changes[corrected_vec_index] =
                                    BufferChanged::Content(*c_i, old.to_string(), new.to_string());

                                continue 'changes;
                            }
                        }
                        BufferChanged::LineAdded(c_i, _) => {
                            if c_i == &index {
                                let max_index = consolidated_changes.len() - 1;
                                let corrected_vec_index = max_index - rev_index;

                                consolidated_changes[corrected_vec_index] =
                                    BufferChanged::LineAdded(*c_i, new.to_string());

                                continue 'changes;
                            } else if &index >= c_i {
                                index -= 1;
                            }
                        }
                        BufferChanged::LineRemoved(c_i, _) => {
                            if &index > c_i {
                                index += 1;
                            }
                        }
                    }
                }

                consolidated_changes.push(change.clone());
            }
            BufferChanged::LineAdded(_, _) => {
                consolidated_changes.push(change.clone());
            }
            BufferChanged::LineRemoved(line_index, _) => {
                let mut index = *line_index;
                for (rev_index, consolidated) in consolidated_changes.iter().rev().enumerate() {
                    match consolidated {
                        BufferChanged::Content(c_i, _, _) => {
                            if c_i == &index {
                                let max_index = consolidated_changes.len() - 1;
                                let corrected_vec_index = max_index - rev_index;

                                consolidated_changes.remove(corrected_vec_index);

                                continue 'changes;
                            }
                        }
                        BufferChanged::LineAdded(c_i, _) => {
                            if c_i == &index {
                                let max_index = consolidated_changes.len() - 1;
                                let corrected_vec_index = max_index - rev_index;

                                consolidated_changes.remove(corrected_vec_index);

                                continue 'changes;
                            } else if &index >= c_i {
                                index -= 1;
                            }
                        }
                        BufferChanged::LineRemoved(c_i, _) => {
                            if &index > c_i {
                                index += 1;
                            }
                        }
                    }
                }

                consolidated_changes.push(change.clone());
            }
        };
    }

    consolidated_changes
}

mod test {
    #[test]
    fn get_uncommited_changes() {
        use crate::model::undo::BufferChanged;

        let mut undo = super::Undo::default();
        let changes = undo.save();
        assert_eq!(changes, vec![]);

        let mut undo = super::Undo::default();
        undo.add(
            &crate::model::Mode::Insert,
            vec![
                BufferChanged::LineAdded(0, "a".to_string()),
                BufferChanged::LineRemoved(4, "m".to_string()),
            ],
        );

        let changes = undo.get_uncommited_changes();
        assert_eq!(changes, Vec::new());

        undo.close_transaction();
        let changes = undo.get_uncommited_changes();
        assert_eq!(
            changes,
            vec![
                BufferChanged::LineAdded(0, "a".to_string()),
                BufferChanged::LineRemoved(4, "m".to_string()),
            ]
        );

        undo.add(
            &crate::model::Mode::Normal,
            vec![BufferChanged::LineAdded(2, "h".to_string())],
        );
        let changes = undo.get_uncommited_changes();
        assert_eq!(
            changes,
            vec![
                BufferChanged::LineAdded(0, "a".to_string()),
                BufferChanged::LineRemoved(4, "m".to_string()),
                BufferChanged::LineAdded(2, "h".to_string()),
            ]
        );

        undo.add(
            &crate::model::Mode::Insert,
            vec![BufferChanged::LineRemoved(5, "m".to_string())],
        );
        let changes = undo.save();
        assert_eq!(
            changes,
            vec![
                BufferChanged::LineAdded(0, "a".to_string()),
                BufferChanged::LineRemoved(4, "m".to_string()),
                BufferChanged::LineAdded(2, "h".to_string()),
                BufferChanged::LineRemoved(5, "m".to_string()),
            ]
        );

        undo.add(
            &crate::model::Mode::Normal,
            vec![BufferChanged::LineAdded(2, "s".to_string())],
        );
        let changes = undo.get_uncommited_changes();
        assert_eq!(changes, vec![BufferChanged::LineAdded(2, "s".to_string()),]);

        let changes = undo.save();
        assert_eq!(changes, vec![BufferChanged::LineAdded(2, "s".to_string()),]);

        let changes = undo.save();
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn consolidate() {
        use crate::model::undo::BufferChanged;

        let changes = vec![
            BufferChanged::LineAdded(0, "a".to_string()),
            BufferChanged::Content(0, "a".to_string(), "d".to_string()),
            BufferChanged::LineRemoved(0, "d".to_string()),
            BufferChanged::LineAdded(0, "e".to_string()),
            BufferChanged::LineAdded(0, "f".to_string()),
            BufferChanged::LineRemoved(0, "e".to_string()),
            BufferChanged::LineAdded(1, "l".to_string()),
            BufferChanged::LineAdded(2, "g".to_string()),
            BufferChanged::Content(2, "".to_string(), "h".to_string()),
            BufferChanged::Content(3, "i_old".to_string(), "i".to_string()),
            BufferChanged::LineAdded(3, "j".to_string()),
            BufferChanged::LineRemoved(3, "j".to_string()),
            BufferChanged::Content(3, "".to_string(), "k".to_string()),
            BufferChanged::LineRemoved(4, "m".to_string()),
        ];
        let consolidated_changes = super::consolidate_modifications(&changes);

        assert_eq!(
            consolidated_changes,
            vec![
                BufferChanged::LineAdded(0, "e".to_string()),
                BufferChanged::LineAdded(1, "l".to_string()),
                BufferChanged::LineAdded(2, "h".to_string()),
                BufferChanged::Content(3, "i_old".to_string(), "k".to_string()),
                BufferChanged::LineRemoved(4, "m".to_string()),
            ]
        );
    }
}
