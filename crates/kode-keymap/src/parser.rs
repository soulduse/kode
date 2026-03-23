use kode_core::event::{KeyCode, KeyEvent, Modifiers};
use kode_editor::command::Command;

use crate::mode::Mode;
use crate::motion::Motion;
use crate::operator::Operator;
use crate::workspace_keys::{self, WorkspaceAction};

/// Result of parsing a key sequence.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// A complete command was parsed.
    Complete(Action),
    /// More keys are needed (e.g., after 'd', waiting for motion).
    Pending,
    /// Key was not recognized; should be ignored.
    None,
}

/// A resolved action from key input.
#[derive(Debug, Clone)]
pub enum Action {
    Command(Command),
    Motion(Motion),
    OperatorMotion {
        operator: Operator,
        motion: Motion,
        count: usize,
    },
    ChangeMode(Mode),
    RepeatCount(usize),
    CommandLine(String),
    Workspace(WorkspaceAction),
}

/// Stateful key sequence parser.
pub struct KeyParser {
    mode: Mode,
    pending_operator: Option<Operator>,
    count: Option<usize>,
    pending_keys: Vec<KeyEvent>,
    awaiting_workspace_key: bool,
}

impl KeyParser {
    pub fn new() -> Self {
        Self {
            mode: Mode::Insert,
            pending_operator: None,
            count: None,
            pending_keys: Vec::new(),
            awaiting_workspace_key: false,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.reset();
    }

    pub fn parse(&mut self, key: KeyEvent) -> ParseResult {
        // Handle workspace prefix mode (Ctrl-A + key)
        if self.awaiting_workspace_key {
            self.awaiting_workspace_key = false;
            if let Some(action) = workspace_keys::parse_workspace_key(&key) {
                return ParseResult::Complete(Action::Workspace(action));
            }
            return ParseResult::None;
        }

        // Ctrl-A triggers workspace prefix in any mode except Insert
        if !self.mode.is_insert() && workspace_keys::is_prefix(&key) {
            self.awaiting_workspace_key = true;
            return ParseResult::Pending;
        }

        match self.mode {
            Mode::Normal => self.parse_normal(key),
            Mode::Insert => self.parse_insert(key),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => self.parse_visual(key),
            Mode::Command => self.parse_command(key),
            Mode::Replace => self.parse_replace(key),
        }
    }

    fn parse_normal(&mut self, key: KeyEvent) -> ParseResult {
        // Handle count prefix
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() && (self.count.is_some() || c != '0') {
                let digit = c.to_digit(10).unwrap() as usize;
                self.count = Some(self.count.unwrap_or(0) * 10 + digit);
                return ParseResult::Pending;
            }
        }

        let count = self.count.take().unwrap_or(1);

        // Handle pending operator + motion
        if let Some(op) = self.pending_operator.take() {
            if let Some(motion) = self.key_to_motion(key) {
                return ParseResult::Complete(Action::OperatorMotion {
                    operator: op,
                    motion,
                    count,
                });
            }
            // dd, yy, cc — operator doubled = line operation
            if let KeyCode::Char(c) = key.code {
                let same_op = match (op, c) {
                    (Operator::Delete, 'd') => true,
                    (Operator::Yank, 'y') => true,
                    (Operator::Change, 'c') => true,
                    _ => false,
                };
                if same_op {
                    return ParseResult::Complete(Action::Command(Command::DeleteLine));
                }
            }
            return ParseResult::None;
        }

        // Normal mode key handling
        match key.code {
            // Mode changes
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                ParseResult::Complete(Action::ChangeMode(Mode::Insert))
            }
            KeyCode::Char('a') => {
                self.mode = Mode::Insert;
                ParseResult::Complete(Action::Command(Command::MoveRight(1)))
            }
            KeyCode::Char('o') => {
                self.mode = Mode::Insert;
                ParseResult::Complete(Action::Command(Command::NewLine))
            }
            KeyCode::Char('O') => {
                self.mode = Mode::Insert;
                ParseResult::Complete(Action::Command(Command::NewLine))
            }
            KeyCode::Char('v') => {
                self.mode = Mode::Visual;
                ParseResult::Complete(Action::ChangeMode(Mode::Visual))
            }
            KeyCode::Char('V') => {
                self.mode = Mode::VisualLine;
                ParseResult::Complete(Action::ChangeMode(Mode::VisualLine))
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                ParseResult::Complete(Action::ChangeMode(Mode::Command))
            }

            // Operators
            KeyCode::Char('d') => {
                self.pending_operator = Some(Operator::Delete);
                ParseResult::Pending
            }
            KeyCode::Char('y') => {
                self.pending_operator = Some(Operator::Yank);
                ParseResult::Pending
            }
            KeyCode::Char('c') => {
                self.pending_operator = Some(Operator::Change);
                self.mode = Mode::Insert;
                ParseResult::Pending
            }

            // Simple commands
            KeyCode::Char('x') => {
                ParseResult::Complete(Action::Command(Command::DeleteForward))
            }
            KeyCode::Char('p') => {
                ParseResult::Complete(Action::Command(Command::Paste(String::new())))
            }
            KeyCode::Char('u') => ParseResult::Complete(Action::Command(Command::Undo)),
            KeyCode::Char('r') if key.modifiers.contains(Modifiers::CTRL) => {
                ParseResult::Complete(Action::Command(Command::Redo))
            }

            // Motions
            _ => {
                if let Some(motion) = self.key_to_motion(key) {
                    let motion = if count > 1 {
                        match motion {
                            Motion::Down => Motion::LineDown(count),
                            Motion::Up => Motion::LineUp(count),
                            _ => motion,
                        }
                    } else {
                        motion
                    };
                    ParseResult::Complete(Action::Motion(motion))
                } else {
                    ParseResult::None
                }
            }
        }
    }

    fn parse_insert(&mut self, key: KeyEvent) -> ParseResult {
        match key.code {
            KeyCode::Escape => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::ChangeMode(Mode::Normal))
            }
            KeyCode::Char(c) if !key.modifiers.contains(Modifiers::CTRL) && !key.modifiers.contains(Modifiers::SUPER) => {
                ParseResult::Complete(Action::Command(Command::InsertChar(c)))
            }
            KeyCode::Enter => ParseResult::Complete(Action::Command(Command::InsertChar('\n'))),
            KeyCode::Backspace => {
                ParseResult::Complete(Action::Command(Command::DeleteBackward))
            }
            KeyCode::Delete => {
                ParseResult::Complete(Action::Command(Command::DeleteForward))
            }
            KeyCode::Tab => {
                ParseResult::Complete(Action::Command(Command::InsertText("    ".into())))
            }
            KeyCode::Left => ParseResult::Complete(Action::Motion(Motion::Left)),
            KeyCode::Right => ParseResult::Complete(Action::Motion(Motion::Right)),
            KeyCode::Up => ParseResult::Complete(Action::Motion(Motion::Up)),
            KeyCode::Down => ParseResult::Complete(Action::Motion(Motion::Down)),
            _ => ParseResult::None,
        }
    }

    fn parse_visual(&mut self, key: KeyEvent) -> ParseResult {
        match key.code {
            KeyCode::Escape => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::ChangeMode(Mode::Normal))
            }
            KeyCode::Char('d') => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::Command(Command::Cut))
            }
            KeyCode::Char('y') => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::Command(Command::Copy))
            }
            _ => {
                if let Some(motion) = self.key_to_motion(key) {
                    ParseResult::Complete(Action::Motion(motion))
                } else {
                    ParseResult::None
                }
            }
        }
    }

    fn parse_command(&mut self, key: KeyEvent) -> ParseResult {
        match key.code {
            KeyCode::Escape => {
                self.mode = Mode::Normal;
                self.pending_keys.clear();
                ParseResult::Complete(Action::ChangeMode(Mode::Normal))
            }
            KeyCode::Enter => {
                let cmd: String = self
                    .pending_keys
                    .drain(..)
                    .filter_map(|k| {
                        if let KeyCode::Char(c) = k.code {
                            Some(c)
                        } else {
                            None
                        }
                    })
                    .collect();
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::CommandLine(cmd))
            }
            _ => {
                self.pending_keys.push(key);
                ParseResult::Pending
            }
        }
    }

    fn parse_replace(&mut self, key: KeyEvent) -> ParseResult {
        match key.code {
            KeyCode::Escape => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::ChangeMode(Mode::Normal))
            }
            KeyCode::Char(c) => {
                self.mode = Mode::Normal;
                ParseResult::Complete(Action::Command(Command::InsertChar(c)))
            }
            _ => ParseResult::None,
        }
    }

    fn key_to_motion(&self, key: KeyEvent) -> Option<Motion> {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => Some(Motion::Left),
            KeyCode::Char('j') | KeyCode::Down => Some(Motion::Down),
            KeyCode::Char('k') | KeyCode::Up => Some(Motion::Up),
            KeyCode::Char('l') | KeyCode::Right => Some(Motion::Right),
            KeyCode::Char('w') => Some(Motion::WordForward),
            KeyCode::Char('b') => Some(Motion::WordBackward),
            KeyCode::Char('e') => Some(Motion::WordEnd),
            KeyCode::Char('0') => Some(Motion::LineStart),
            KeyCode::Char('$') => Some(Motion::LineEnd),
            KeyCode::Char('^') => Some(Motion::FirstNonBlank),
            KeyCode::Char('g') if key.modifiers == Modifiers::NONE => Some(Motion::FileStart),
            KeyCode::Char('G') => Some(Motion::FileEnd),
            KeyCode::Char('%') => Some(Motion::MatchBracket),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.pending_operator = None;
        self.count = None;
        self.pending_keys.clear();
        self.awaiting_workspace_key = false;
    }
}

impl Default for KeyParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: char) -> KeyEvent {
        KeyEvent::plain(KeyCode::Char(c))
    }

    #[test]
    fn normal_mode_motion() {
        let mut parser = KeyParser::new();
        parser.set_mode(Mode::Normal);
        match parser.parse(key('j')) {
            ParseResult::Complete(Action::Motion(Motion::Down)) => {}
            other => panic!("Expected Motion::Down, got {:?}", other),
        }
    }

    #[test]
    fn insert_mode_char() {
        let mut parser = KeyParser::new();
        parser.set_mode(Mode::Insert);
        match parser.parse(key('a')) {
            ParseResult::Complete(Action::Command(Command::InsertChar('a'))) => {}
            other => panic!("Expected InsertChar('a'), got {:?}", other),
        }
    }

    #[test]
    fn operator_motion() {
        let mut parser = KeyParser::new();
        parser.set_mode(Mode::Normal);
        assert!(matches!(parser.parse(key('d')), ParseResult::Pending));
        match parser.parse(key('w')) {
            ParseResult::Complete(Action::OperatorMotion {
                operator: Operator::Delete,
                motion: Motion::WordForward,
                count: 1,
            }) => {}
            other => panic!("Expected dw, got {:?}", other),
        }
    }

    #[test]
    fn count_prefix() {
        let mut parser = KeyParser::new();
        parser.set_mode(Mode::Normal);
        assert!(matches!(parser.parse(key('3')), ParseResult::Pending));
        match parser.parse(key('j')) {
            ParseResult::Complete(Action::Motion(Motion::LineDown(3))) => {}
            other => panic!("Expected 3j, got {:?}", other),
        }
    }
}
