use std::path::PathBuf;

/// High-level editor commands.
#[derive(Debug, Clone)]
pub enum Command {
    // Editing
    InsertChar(char),
    InsertText(String),
    NewLine,
    DeleteForward,
    DeleteBackward,
    DeleteLine,
    DeleteToLineEnd,

    // Movement
    MoveUp(usize),
    MoveDown(usize),
    MoveLeft(usize),
    MoveRight(usize),
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    PageUp,
    PageDown,
    GotoLine(usize),

    // Selection
    SelectAll,
    SelectLine,
    SelectWord,

    // Clipboard
    Copy,
    Cut,
    Paste(String),
    Yank { text: String },

    // History
    Undo,
    Redo,

    // File
    Save,
    SaveAs(PathBuf),
    Open(PathBuf),

    // Search
    Find(String),
    FindNext,
    FindPrev,
    Replace { find: String, replace: String },
    ReplaceAll { find: String, replace: String },

    // Application
    Quit,
    ForceQuit,
}
