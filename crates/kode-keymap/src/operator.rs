use kode_core::geometry::Position;
use kode_editor::command::Command;

/// Vim operators that act on a motion range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
    Indent,
    Unindent,
}

impl Operator {
    /// Convert an operator + range into editor commands.
    pub fn to_commands(self, start: Position, end: Position) -> Vec<Command> {
        let (from, to) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        match self {
            Operator::Delete => {
                vec![Command::DeleteForward] // Simplified; real impl needs range delete
            }
            Operator::Yank => {
                vec![Command::Copy]
            }
            Operator::Change => {
                vec![Command::DeleteForward] // Delete then enter insert mode
            }
            Operator::Indent => {
                // Insert tab at start of each line in range
                let mut cmds = Vec::new();
                for line in from.line..=to.line {
                    cmds.push(Command::InsertText("    ".into()));
                }
                cmds
            }
            Operator::Unindent => {
                let mut cmds = Vec::new();
                for line in from.line..=to.line {
                    cmds.push(Command::DeleteBackward);
                }
                cmds
            }
        }
    }
}
