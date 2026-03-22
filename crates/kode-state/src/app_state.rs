use std::collections::HashMap;

use kode_core::geometry::Rect;
use kode_editor::command::Command;
use kode_editor::document::Document;
use kode_keymap::mode::Mode;
use kode_keymap::motion::Motion;
use kode_keymap::parser::{Action, KeyParser, ParseResult};
use kode_keymap::workspace_keys::WorkspaceAction;
use kode_terminal::Terminal;
use kode_workspace::layout::{Direction, FocusDirection};
use kode_workspace::pane::{Pane, PaneContent, PaneId};
use kode_workspace::session::Session;
use kode_workspace::tab::Tab;

use crate::file_explorer::FileExplorer;

/// Shared app state used by both TUI and GPU modes.
pub struct AppState {
    pub documents: HashMap<usize, Document>,
    pub terminals: HashMap<usize, Terminal>,
    pub explorers: HashMap<usize, FileExplorer>,
    pub panes: HashMap<PaneId, Pane>,
    pub session: Session,
    pub key_parser: KeyParser,
    pub focused_pane: PaneId,
    pub viewport: Rect,
    running: bool,
    zoomed_pane: Option<PaneId>,
    command_buffer: String,
}

impl AppState {
    pub fn set_viewport(&mut self, cols: u16, rows: u16) {
        let h = (rows as f32).max(2.0) - 2.0;
        self.viewport = Rect::new(0.0, 0.0, cols as f32, h);
    }

    pub fn set_viewport_pixels(&mut self, width: f32, height: f32) {
        self.viewport = Rect::new(0.0, 0.0, width, height);
    }

    pub fn mode(&self) -> Mode {
        self.key_parser.mode()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn command_text(&self) -> &str {
        &self.command_buffer
    }

    pub fn pane_rects(&self) -> Vec<(PaneId, Rect)> {
        if let Some(zoomed) = self.zoomed_pane {
            vec![(zoomed, self.viewport)]
        } else {
            self.session
                .active_tab()
                .layout
                .compute_rects(self.viewport)
        }
    }

    pub fn handle_key_event(&mut self, key: kode_core::event::KeyEvent) {
        // In command mode, accumulate text in our buffer
        if self.key_parser.mode() == Mode::Command {
            match key.code {
                kode_core::event::KeyCode::Escape => {
                    self.command_buffer.clear();
                    self.key_parser.parse(key);
                }
                kode_core::event::KeyCode::Enter => {
                    let cmd = self.command_buffer.clone();
                    self.command_buffer.clear();
                    self.key_parser.parse(key);
                    self.handle_command_line(&cmd);
                }
                kode_core::event::KeyCode::Backspace => {
                    self.command_buffer.pop();
                    if self.command_buffer.is_empty() {
                        self.key_parser.parse(kode_core::event::KeyEvent::plain(
                            kode_core::event::KeyCode::Escape,
                        ));
                    } else {
                        self.key_parser.parse(key);
                    }
                }
                kode_core::event::KeyCode::Char(c) => {
                    self.command_buffer.push(c);
                    self.key_parser.parse(key);
                }
                _ => {
                    self.key_parser.parse(key);
                }
            }
            return;
        }

        match self.key_parser.parse(key) {
            ParseResult::Complete(Action::Workspace(action)) => {
                self.handle_workspace_action(action);
            }
            ParseResult::Complete(Action::Command(cmd)) => {
                self.handle_editor_command(cmd);
            }
            ParseResult::Complete(Action::Motion(motion)) => {
                self.handle_motion(motion);
            }
            ParseResult::Complete(Action::ChangeMode(mode)) => {
                if mode == Mode::Command {
                    self.command_buffer.clear();
                }
            }
            ParseResult::Complete(Action::CommandLine(cmd)) => {
                self.handle_command_line(&cmd);
            }
            ParseResult::Complete(_) => {}
            ParseResult::Pending => {}
            ParseResult::None => {}
        }
    }

    fn handle_editor_command(&mut self, cmd: Command) {
        let doc = match self.focused_editor_doc_mut() {
            Some(d) => d,
            None => return,
        };

        match cmd {
            Command::InsertChar(ch) => doc.insert_char(ch),
            Command::InsertText(text) => doc.insert_text(&text),
            Command::DeleteBackward => doc.delete_backward(),
            Command::DeleteForward => doc.delete_forward(),
            Command::DeleteLine => {
                let line = doc.cursors.primary().line();
                let start = doc.buffer.line_to_char(line);
                let end = if line + 1 < doc.buffer.line_count() {
                    doc.buffer.line_to_char(line + 1)
                } else {
                    doc.buffer.char_count()
                };
                if end > start {
                    let op = doc.buffer.delete(start..end);
                    doc.history.record(op);
                }
            }
            Command::Undo => doc.undo(),
            Command::Redo => doc.redo(),
            Command::MoveRight(n) => {
                let line = doc.cursors.primary().line();
                let col = doc.cursors.primary().col() + n;
                let max_col = doc.buffer.line_len(line);
                doc.cursors.primary_mut().move_to(line, col.min(max_col));
            }
            Command::NewLine => doc.insert_char('\n'),
            _ => {}
        }
    }

    fn handle_motion(&mut self, motion: Motion) {
        let doc = match self.focused_editor_doc_mut() {
            Some(d) => d,
            None => return,
        };

        let line = doc.cursors.primary().line();
        let col = doc.cursors.primary().col();

        let (new_line, new_col) = match motion {
            Motion::Left => (line, col.saturating_sub(1)),
            Motion::Right => {
                let max = doc.buffer.line_len(line);
                (line, (col + 1).min(max))
            }
            Motion::Up => (line.saturating_sub(1), col),
            Motion::Down => {
                let max = doc.buffer.line_count().saturating_sub(1);
                ((line + 1).min(max), col)
            }
            Motion::LineUp(n) => (line.saturating_sub(n), col),
            Motion::LineDown(n) => {
                let max = doc.buffer.line_count().saturating_sub(1);
                ((line + n).min(max), col)
            }
            Motion::LineStart => (line, 0),
            Motion::LineEnd => (line, doc.buffer.line_len(line)),
            Motion::FirstNonBlank => {
                let fnb = doc
                    .buffer
                    .line_to_string(line)
                    .map(|s| s.find(|c: char| !c.is_whitespace()).unwrap_or(0))
                    .unwrap_or(0);
                (line, fnb)
            }
            Motion::FileStart => (0, 0),
            Motion::FileEnd => (doc.buffer.line_count().saturating_sub(1), 0),
            Motion::WordForward => {
                let text = doc.buffer.line_to_string(line).unwrap_or_default();
                let rest = &text[col..];
                let skip = rest
                    .find(|c: char| c.is_whitespace())
                    .and_then(|ws| rest[ws..].find(|c: char| !c.is_whitespace()).map(|nws| ws + nws))
                    .unwrap_or(rest.len());
                if col + skip >= text.len() && line + 1 < doc.buffer.line_count() {
                    (line + 1, 0)
                } else {
                    (line, col + skip)
                }
            }
            Motion::WordBackward => {
                if col == 0 && line > 0 {
                    (line - 1, doc.buffer.line_len(line - 1))
                } else if col == 0 {
                    (0, 0)
                } else {
                    let text = doc.buffer.line_to_string(line).unwrap_or_default();
                    let before = &text[..col];
                    let nc = before.rfind(|c: char| c.is_whitespace()).map(|p| p + 1).unwrap_or(0);
                    (line, nc)
                }
            }
            Motion::WordEnd => {
                let text = doc.buffer.line_to_string(line).unwrap_or_default();
                let start = (col + 1).min(text.len());
                let rest = &text[start..];
                let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
                (line, (start + end).min(text.len()))
            }
            _ => (line, col),
        };

        let max_col = doc.buffer.line_len(new_line);
        doc.cursors.primary_mut().move_to(new_line, new_col.min(max_col));
    }

    fn handle_workspace_action(&mut self, action: WorkspaceAction) {
        match action {
            WorkspaceAction::SplitVertical => self.split_pane(Direction::Vertical),
            WorkspaceAction::SplitHorizontal => self.split_pane(Direction::Horizontal),
            WorkspaceAction::FocusLeft => self.focus_direction(FocusDirection::Left),
            WorkspaceAction::FocusRight => self.focus_direction(FocusDirection::Right),
            WorkspaceAction::FocusUp => self.focus_direction(FocusDirection::Up),
            WorkspaceAction::FocusDown => self.focus_direction(FocusDirection::Down),
            WorkspaceAction::ClosePane => self.close_focused_pane(),
            WorkspaceAction::ZoomPane => {
                self.zoomed_pane = if self.zoomed_pane.is_some() { None } else { Some(self.focused_pane) };
            }
            WorkspaceAction::NewTerminal => self.spawn_terminal(),
            WorkspaceAction::NewTab => self.new_tab(),
            WorkspaceAction::NextTab => self.session.next_tab(),
            WorkspaceAction::PrevTab => self.session.prev_tab(),
            WorkspaceAction::CloseTab => {
                let idx = self.session.active_tab;
                self.session.close_tab(idx);
            }
            WorkspaceAction::ToggleExplorer => self.toggle_explorer(),
            WorkspaceAction::Detach => self.running = false,
            WorkspaceAction::Equalize => {
                kode_workspace::resize::equalize_panes(&mut self.session.active_tab_mut().layout);
            }
            WorkspaceAction::ResizeLeft(_) | WorkspaceAction::ResizeUp(_) => {
                kode_workspace::resize::resize_pane(
                    &mut self.session.active_tab_mut().layout, self.focused_pane, -0.05,
                );
            }
            WorkspaceAction::ResizeRight(_) | WorkspaceAction::ResizeDown(_) => {
                kode_workspace::resize::resize_pane(
                    &mut self.session.active_tab_mut().layout, self.focused_pane, 0.05,
                );
            }
        }
    }

    fn handle_command_line(&mut self, cmd: &str) {
        match cmd {
            "w" => {
                if let Some(doc) = self.focused_editor_doc_mut() {
                    let _ = doc.save();
                }
            }
            "q" => self.running = false,
            "wq" => {
                if let Some(doc) = self.focused_editor_doc_mut() {
                    let _ = doc.save();
                }
                self.running = false;
            }
            "explorer" => self.toggle_explorer(),
            _ => tracing::warn!("Unknown command: :{}", cmd),
        }
    }

    fn focused_editor_doc_mut(&mut self) -> Option<&mut Document> {
        let pane = self.panes.get(&self.focused_pane)?;
        match pane.content {
            PaneContent::Editor(doc_id) => self.documents.get_mut(&doc_id),
            _ => None,
        }
    }

    fn split_pane(&mut self, direction: Direction) {
        let new_doc_id = self.documents.keys().max().copied().unwrap_or(0) + 1;
        self.documents.insert(new_doc_id, Document::new());
        let new_pane_id = self.panes.keys().max().copied().unwrap_or(0) + 1;
        self.panes.insert(new_pane_id, Pane::editor(new_pane_id, new_doc_id));
        self.session.active_tab_mut().layout.split(self.focused_pane, new_pane_id, direction);
        self.set_focus(new_pane_id);
    }

    fn spawn_terminal(&mut self) {
        let term_id = self.terminals.keys().max().copied().unwrap_or(0) + 1;
        match Terminal::spawn(term_id, 24, 80) {
            Ok(terminal) => {
                self.terminals.insert(term_id, terminal);
                let pane_id = self.panes.keys().max().copied().unwrap_or(0) + 1;
                self.panes.insert(pane_id, Pane::terminal(pane_id, term_id));
                self.session.active_tab_mut().layout.split(self.focused_pane, pane_id, Direction::Horizontal);
                self.set_focus(pane_id);
            }
            Err(e) => tracing::error!("Failed to spawn terminal: {}", e),
        }
    }

    fn new_tab(&mut self) {
        let doc_id = self.documents.keys().max().copied().unwrap_or(0) + 1;
        self.documents.insert(doc_id, Document::new());
        let pane_id = self.panes.keys().max().copied().unwrap_or(0) + 1;
        self.panes.insert(pane_id, Pane::editor(pane_id, doc_id));
        let tab_id = self.session.tabs.len();
        self.session.add_tab(Tab::new(tab_id, format!("tab-{}", tab_id), pane_id));
        self.set_focus(pane_id);
    }

    fn focus_direction(&mut self, dir: FocusDirection) {
        let rects = self.session.active_tab().layout.compute_rects(self.viewport);
        if let Some(target) = kode_workspace::layout::find_pane_in_direction(self.focused_pane, dir, &rects) {
            self.set_focus(target);
        }
    }

    fn close_focused_pane(&mut self) {
        let layout = &mut self.session.active_tab_mut().layout;
        if layout.pane_ids().len() <= 1 {
            return;
        }
        let rects = layout.compute_rects(self.viewport);
        let next = kode_workspace::layout::find_pane_in_direction(self.focused_pane, FocusDirection::Left, &rects)
            .or_else(|| kode_workspace::layout::find_pane_in_direction(self.focused_pane, FocusDirection::Right, &rects));
        let removed = self.focused_pane;
        layout.remove(removed);
        if let Some(pane) = self.panes.remove(&removed) {
            match pane.content {
                PaneContent::Editor(doc_id) => { self.documents.remove(&doc_id); }
                PaneContent::Terminal(term_id) => { self.terminals.remove(&term_id); }
                PaneContent::FileExplorer(explorer_id) => { self.explorers.remove(&explorer_id); }
                _ => {}
            }
        }
        if let Some(next) = next {
            self.set_focus(next);
        } else if let Some(&first) = self.session.active_tab().layout.pane_ids().first() {
            self.set_focus(first);
        }
    }

    fn set_focus(&mut self, pane_id: PaneId) {
        if let Some(old) = self.panes.get_mut(&self.focused_pane) { old.focused = false; }
        if let Some(new) = self.panes.get_mut(&pane_id) { new.focused = true; }
        self.focused_pane = pane_id;
    }

    /// Toggle the file explorer pane.
    pub fn toggle_explorer(&mut self) {
        let existing = self.panes.iter().find(|(_, p)| {
            matches!(p.content, PaneContent::FileExplorer(_))
        }).map(|(&id, _)| id);

        if let Some(explorer_pane_id) = existing {
            let layout = &mut self.session.active_tab_mut().layout;
            if layout.pane_ids().len() <= 1 {
                return;
            }
            layout.remove(explorer_pane_id);
            if let Some(pane) = self.panes.remove(&explorer_pane_id) {
                if let PaneContent::FileExplorer(eid) = pane.content {
                    self.explorers.remove(&eid);
                }
            }
            if self.focused_pane == explorer_pane_id {
                if let Some(&first) = self.session.active_tab().layout.pane_ids().first() {
                    self.set_focus(first);
                }
            }
        } else {
            self.spawn_explorer();
        }
    }

    /// Spawn a new file explorer pane on the left.
    pub fn spawn_explorer(&mut self) {
        let explorer_id = self.explorers.keys().max().copied().unwrap_or(0) + 1;
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        self.explorers.insert(explorer_id, FileExplorer::new(explorer_id, root));

        let pane_id = self.panes.keys().max().copied().unwrap_or(0) + 1;
        self.panes.insert(pane_id, Pane::file_explorer(pane_id, explorer_id));

        let layout = &mut self.session.active_tab_mut().layout;
        let old_layout = std::mem::replace(layout, kode_workspace::layout::LayoutNode::Leaf(pane_id));
        *layout = kode_workspace::layout::LayoutNode::Split {
            direction: Direction::Vertical,
            ratio: 0.25,
            first: Box::new(kode_workspace::layout::LayoutNode::Leaf(pane_id)),
            second: Box::new(old_layout),
        };
        self.set_focus(pane_id);
    }

    /// Open a file from the explorer in the nearest editor pane.
    pub fn open_file_from_explorer(&mut self, path: std::path::PathBuf) {
        let editor_pane = self.panes.iter().find(|(_, p)| {
            matches!(p.content, PaneContent::Editor(_))
        }).map(|(&id, p)| (id, match p.content { PaneContent::Editor(did) => did, _ => 0 }));

        if let Some((pane_id, doc_id)) = editor_pane {
            match Document::from_file(path) {
                Ok(doc) => {
                    self.documents.insert(doc_id, doc);
                    self.set_focus(pane_id);
                }
                Err(e) => tracing::error!("Failed to open file: {}", e),
            }
        } else {
            let doc_id = self.documents.keys().max().copied().unwrap_or(0) + 1;
            match Document::from_file(path) {
                Ok(doc) => {
                    self.documents.insert(doc_id, doc);
                    let pane_id = self.panes.keys().max().copied().unwrap_or(0) + 1;
                    self.panes.insert(pane_id, Pane::editor(pane_id, doc_id));
                    self.session.active_tab_mut().layout.split(
                        self.focused_pane, pane_id, Direction::Vertical,
                    );
                    self.set_focus(pane_id);
                }
                Err(e) => tracing::error!("Failed to open file: {}", e),
            }
        }
    }
}

/// Create an AppState from initial parameters.
pub fn create_app_state(
    documents: HashMap<usize, Document>,
    terminals: HashMap<usize, Terminal>,
    explorers: HashMap<usize, FileExplorer>,
    panes: HashMap<PaneId, Pane>,
    session: Session,
    focused_pane: PaneId,
) -> AppState {
    AppState {
        documents,
        terminals,
        explorers,
        panes,
        session,
        key_parser: KeyParser::new(),
        focused_pane,
        viewport: Rect::new(0.0, 0.0, 80.0, 24.0),
        running: true,
        zoomed_pane: None,
        command_buffer: String::new(),
    }
}
