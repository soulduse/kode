use std::path::PathBuf;

use crate::buffer::Buffer;
use crate::cursor::CursorSet;
use crate::history::History;
use crate::selection::SelectionSet;

/// A document ties together all editing state for a single file.
#[derive(Debug)]
pub struct Document {
    pub buffer: Buffer,
    pub cursors: CursorSet,
    pub selections: SelectionSet,
    pub history: History,
    pub file_path: Option<PathBuf>,
    pub language: Option<String>,
    scroll_offset: usize,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursors: CursorSet::new(),
            selections: SelectionSet::new(),
            history: History::new(),
            file_path: None,
            language: None,
            scroll_offset: 0,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let buffer = Buffer::from_file(&path)?;
        let language = detect_language(&path);
        Ok(Self {
            buffer,
            cursors: CursorSet::new(),
            selections: SelectionSet::new(),
            history: History::new(),
            file_path: Some(path),
            language,
            scroll_offset: 0,
        })
    }

    pub fn insert_char(&mut self, ch: char) {
        let pos = self.cursor_to_char_idx();
        let op = self.buffer.insert(pos, &ch.to_string());
        self.history.record(op);

        // Advance cursor
        if ch == '\n' {
            let line = self.cursors.primary().line();
            self.cursors.primary_mut().move_to(line + 1, 0);
        } else {
            let line = self.cursors.primary().line();
            let col = self.cursors.primary().col();
            self.cursors.primary_mut().move_to(line, col + 1);
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        let pos = self.cursor_to_char_idx();
        let op = self.buffer.insert(pos, text);
        self.history.record(op);
    }

    pub fn delete_backward(&mut self) {
        let pos = self.cursor_to_char_idx();
        if pos == 0 {
            return;
        }
        let op = self.buffer.delete(pos - 1..pos);
        self.history.record(op);

        // Move cursor back
        let cursor = self.cursors.primary();
        if cursor.col() > 0 {
            let col = cursor.col() - 1;
            let line = cursor.line();
            self.cursors.primary_mut().move_to(line, col);
        } else if cursor.line() > 0 {
            let line = cursor.line() - 1;
            let col = self.buffer.line_len(line);
            self.cursors.primary_mut().move_to(line, col);
        }
    }

    pub fn delete_forward(&mut self) {
        let pos = self.cursor_to_char_idx();
        if pos >= self.buffer.char_count() {
            return;
        }
        let op = self.buffer.delete(pos..pos + 1);
        self.history.record(op);
    }

    pub fn commit_edit(&mut self) {
        self.history.commit();
    }

    pub fn undo(&mut self) {
        if let Some(txn) = self.history.undo() {
            for op in txn.operations().iter().rev() {
                self.buffer.apply_reverse(op);
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(txn) = self.history.redo() {
            for op in txn.operations() {
                self.buffer.apply(op);
            }
        }
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.file_path {
            std::fs::write(path, self.buffer.text())?;
            self.buffer.set_unmodified();
        }
        Ok(())
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    pub fn title(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "[untitled]".into())
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    fn cursor_to_char_idx(&self) -> usize {
        let cursor = self.cursors.primary();
        let line_start = self.buffer.line_to_char(cursor.line());
        line_start + cursor.col()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_language(path: &std::path::Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext {
            "kt" | "kts" => "kotlin",
            "java" => "java",
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "tsx" => "typescriptreact",
            "yml" | "yaml" => "yaml",
            "toml" => "toml",
            "json" => "json",
            "md" => "markdown",
            "groovy" | "gradle" => "groovy",
            _ => "plaintext",
        })
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_undo() {
        let mut doc = Document::new();
        doc.insert_char('h');
        doc.insert_char('i');
        doc.commit_edit();
        assert_eq!(doc.buffer.text(), "hi");

        doc.undo();
        assert_eq!(doc.buffer.text(), "");
    }

    #[test]
    fn detect_kotlin() {
        let lang = detect_language(std::path::Path::new("Main.kt"));
        assert_eq!(lang, Some("kotlin".into()));
    }

    #[test]
    fn title_untitled() {
        let doc = Document::new();
        assert_eq!(doc.title(), "[untitled]");
    }

    #[test]
    fn title_with_path() {
        let mut doc = Document::new();
        doc.file_path = Some(PathBuf::from("/some/path/Main.kt"));
        assert_eq!(doc.title(), "Main.kt");
    }
}
