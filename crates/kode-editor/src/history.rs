/// A single edit operation (insert or delete).
#[derive(Debug, Clone, PartialEq)]
pub enum EditOperation {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
}

/// A transaction groups multiple edits into one undoable unit.
#[derive(Debug, Clone)]
pub struct Transaction {
    operations: Vec<EditOperation>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn push(&mut self, op: EditOperation) {
        self.operations.push(op);
    }

    pub fn operations(&self) -> &[EditOperation] {
        &self.operations
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Linear undo/redo history.
#[derive(Debug)]
pub struct History {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    current: Option<Transaction>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current: None,
        }
    }

    /// Record an edit operation into the current transaction.
    pub fn record(&mut self, op: EditOperation) {
        if self.current.is_none() {
            self.current = Some(Transaction::new());
        }
        self.current.as_mut().unwrap().push(op);
    }

    /// Commit the current transaction to the undo stack.
    pub fn commit(&mut self) {
        if let Some(txn) = self.current.take() {
            if !txn.is_empty() {
                self.undo_stack.push(txn);
                self.redo_stack.clear();
            }
        }
    }

    /// Undo the last transaction. Returns operations to reverse.
    pub fn undo(&mut self) -> Option<&Transaction> {
        self.commit();
        if let Some(txn) = self.undo_stack.pop() {
            self.redo_stack.push(txn);
            self.redo_stack.last()
        } else {
            None
        }
    }

    /// Redo the last undone transaction. Returns operations to apply.
    pub fn redo(&mut self) -> Option<&Transaction> {
        if let Some(txn) = self.redo_stack.pop() {
            self.undo_stack.push(txn);
            self.undo_stack.last()
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || self.current.is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo() {
        let mut history = History::new();

        history.record(EditOperation::Insert {
            pos: 0,
            text: "hello".into(),
        });
        history.commit();

        history.record(EditOperation::Insert {
            pos: 5,
            text: " world".into(),
        });
        history.commit();

        assert!(history.can_undo());
        let txn = history.undo().unwrap();
        assert_eq!(txn.operations().len(), 1);

        assert!(history.can_redo());
        let txn = history.redo().unwrap();
        assert_eq!(txn.operations().len(), 1);
    }

    #[test]
    fn redo_cleared_on_new_edit() {
        let mut history = History::new();
        history.record(EditOperation::Insert {
            pos: 0,
            text: "a".into(),
        });
        history.commit();

        history.undo();
        assert!(history.can_redo());

        history.record(EditOperation::Insert {
            pos: 0,
            text: "b".into(),
        });
        history.commit();
        assert!(!history.can_redo());
    }
}
