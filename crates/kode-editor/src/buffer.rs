use ropey::Rope;
use std::ops::Range;

use crate::history::EditOperation;

/// Rope-based text buffer optimized for large files.
#[derive(Debug, Clone)]
pub struct Buffer {
    rope: Rope,
    modified: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            modified: false,
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            modified: false,
        }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self {
            rope: Rope::from_str(&text),
            modified: false,
        })
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) -> EditOperation {
        let op = EditOperation::Insert {
            pos: char_idx,
            text: text.to_string(),
        };
        self.rope.insert(char_idx, text);
        self.modified = true;
        op
    }

    pub fn delete(&mut self, range: Range<usize>) -> EditOperation {
        let deleted: String = self.rope.slice(range.clone()).into();
        let op = EditOperation::Delete {
            pos: range.start,
            text: deleted,
        };
        self.rope.remove(range);
        self.modified = true;
        op
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn line(&self, idx: usize) -> Option<ropey::RopeSlice<'_>> {
        if idx < self.line_count() {
            Some(self.rope.line(idx))
        } else {
            None
        }
    }

    pub fn line_to_string(&self, idx: usize) -> Option<String> {
        self.line(idx).map(|l| {
            let s: String = l.into();
            s.trim_end_matches('\n').to_string()
        })
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.line_count() {
            return 0;
        }
        let line = self.rope.line(line_idx);
        let len = line.len_chars();
        // Subtract trailing newline if present
        if len > 0 {
            let last = line.char(len - 1);
            if last == '\n' {
                return len - 1;
            }
        }
        len
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn set_unmodified(&mut self) {
        self.modified = false;
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Apply an edit operation (used for undo/redo).
    pub fn apply(&mut self, op: &EditOperation) {
        match op {
            EditOperation::Insert { pos, text } => {
                self.rope.insert(*pos, text);
            }
            EditOperation::Delete { pos, text } => {
                self.rope.remove(*pos..*pos + text.len());
            }
        }
        self.modified = true;
    }

    /// Apply the reverse of an edit operation.
    pub fn apply_reverse(&mut self, op: &EditOperation) {
        match op {
            EditOperation::Insert { pos, text } => {
                self.rope.remove(*pos..*pos + text.len());
            }
            EditOperation::Delete { pos, text } => {
                self.rope.insert(*pos, text);
            }
        }
        self.modified = true;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_delete() {
        let mut buf = Buffer::from_str("hello world");
        buf.insert(5, " beautiful");
        assert_eq!(buf.text(), "hello beautiful world");

        buf.delete(5..15);
        assert_eq!(buf.text(), "hello world");
    }

    #[test]
    fn line_operations() {
        let buf = Buffer::from_str("line one\nline two\nline three");
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line_to_string(0), Some("line one".into()));
        assert_eq!(buf.line_to_string(1), Some("line two".into()));
        assert_eq!(buf.line_len(0), 8);
    }

    #[test]
    fn from_file_missing() {
        let result = Buffer::from_file(std::path::Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn modified_tracking() {
        let mut buf = Buffer::new();
        assert!(!buf.is_modified());
        buf.insert(0, "hello");
        assert!(buf.is_modified());
        buf.set_unmodified();
        assert!(!buf.is_modified());
    }
}
