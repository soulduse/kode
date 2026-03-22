use kode_core::geometry::Position;
use kode_editor::buffer::Buffer;

/// A motion describes a cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    WordEnd,
    LineStart,
    LineEnd,
    FirstNonBlank,
    FileStart,
    FileEnd,
    LineUp(usize),
    LineDown(usize),
    FindChar(char),
    FindCharBackward(char),
    MatchBracket,
}

impl Motion {
    /// Apply this motion to a position, returning the new position.
    pub fn apply(&self, pos: Position, buffer: &Buffer) -> Position {
        match self {
            Motion::Left => move_left(pos),
            Motion::Right => move_right(pos, buffer),
            Motion::Up => move_up(pos),
            Motion::Down => move_down(pos, buffer),
            Motion::WordForward => word_forward(pos, buffer),
            Motion::WordBackward => word_backward(pos, buffer),
            Motion::WordEnd => word_end(pos, buffer),
            Motion::LineStart => Position::new(pos.line, 0),
            Motion::LineEnd => {
                let len = buffer.line_len(pos.line);
                Position::new(pos.line, len.saturating_sub(1).max(0))
            }
            Motion::FirstNonBlank => first_non_blank(pos, buffer),
            Motion::FileStart => Position::new(0, 0),
            Motion::FileEnd => {
                let last = buffer.line_count().saturating_sub(1);
                Position::new(last, 0)
            }
            Motion::LineUp(n) => Position::new(pos.line.saturating_sub(*n), pos.col),
            Motion::LineDown(n) => {
                let line = (pos.line + n).min(buffer.line_count().saturating_sub(1));
                Position::new(line, pos.col)
            }
            Motion::FindChar(ch) => find_char_forward(pos, buffer, *ch),
            Motion::FindCharBackward(ch) => find_char_backward(pos, buffer, *ch),
            Motion::MatchBracket => match_bracket(pos, buffer),
        }
    }
}

fn move_left(pos: Position) -> Position {
    Position::new(pos.line, pos.col.saturating_sub(1))
}

fn move_right(pos: Position, buffer: &Buffer) -> Position {
    let line_len = buffer.line_len(pos.line);
    let max_col = if line_len > 0 { line_len - 1 } else { 0 };
    Position::new(pos.line, (pos.col + 1).min(max_col))
}

fn move_up(pos: Position) -> Position {
    Position::new(pos.line.saturating_sub(1), pos.col)
}

fn move_down(pos: Position, buffer: &Buffer) -> Position {
    let max_line = buffer.line_count().saturating_sub(1);
    Position::new((pos.line + 1).min(max_line), pos.col)
}

fn word_forward(pos: Position, buffer: &Buffer) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    let mut col = pos.col;

    // Skip current word characters
    while col < chars.len() && is_word_char(chars[col]) {
        col += 1;
    }
    // Skip whitespace
    while col < chars.len() && chars[col].is_whitespace() {
        col += 1;
    }

    if col >= chars.len() {
        // Move to next line
        if pos.line + 1 < buffer.line_count() {
            return first_non_blank(Position::new(pos.line + 1, 0), buffer);
        }
        return Position::new(pos.line, chars.len().saturating_sub(1));
    }

    Position::new(pos.line, col)
}

fn word_backward(pos: Position, buffer: &Buffer) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    let mut col = pos.col;

    if col == 0 {
        if pos.line > 0 {
            let prev_len = buffer.line_len(pos.line - 1);
            return Position::new(pos.line - 1, prev_len.saturating_sub(1));
        }
        return pos;
    }

    col -= 1;
    // Skip whitespace backwards
    while col > 0 && chars[col].is_whitespace() {
        col -= 1;
    }
    // Skip word chars backwards
    while col > 0 && is_word_char(chars[col - 1]) {
        col -= 1;
    }

    Position::new(pos.line, col)
}

fn word_end(pos: Position, buffer: &Buffer) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    let mut col = pos.col + 1;

    // Skip whitespace
    while col < chars.len() && chars[col].is_whitespace() {
        col += 1;
    }
    // Advance to end of word
    while col + 1 < chars.len() && is_word_char(chars[col + 1]) {
        col += 1;
    }

    if col >= chars.len() {
        return Position::new(pos.line, chars.len().saturating_sub(1));
    }

    Position::new(pos.line, col)
}

fn first_non_blank(pos: Position, buffer: &Buffer) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let col = line_str
        .chars()
        .position(|c| !c.is_whitespace())
        .unwrap_or(0);
    Position::new(pos.line, col)
}

fn find_char_forward(pos: Position, buffer: &Buffer, ch: char) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    for i in (pos.col + 1)..chars.len() {
        if chars[i] == ch {
            return Position::new(pos.line, i);
        }
    }
    pos
}

fn find_char_backward(pos: Position, buffer: &Buffer, ch: char) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    for i in (0..pos.col).rev() {
        if chars[i] == ch {
            return Position::new(pos.line, i);
        }
    }
    pos
}

fn match_bracket(pos: Position, buffer: &Buffer) -> Position {
    let Some(line_str) = buffer.line_to_string(pos.line) else {
        return pos;
    };
    let chars: Vec<char> = line_str.chars().collect();
    if pos.col >= chars.len() {
        return pos;
    }

    let ch = chars[pos.col];
    let (target, forward) = match ch {
        '(' => (')', true),
        '[' => (']', true),
        '{' => ('}', true),
        ')' => ('(', false),
        ']' => ('[', false),
        '}' => ('{', false),
        _ => return pos,
    };

    // Simple single-line bracket matching
    let mut depth = 0i32;
    if forward {
        for i in pos.col..chars.len() {
            if chars[i] == ch {
                depth += 1;
            }
            if chars[i] == target {
                depth -= 1;
            }
            if depth == 0 {
                return Position::new(pos.line, i);
            }
        }
    } else {
        for i in (0..=pos.col).rev() {
            if chars[i] == ch {
                depth += 1;
            }
            if chars[i] == target {
                depth -= 1;
            }
            if depth == 0 {
                return Position::new(pos.line, i);
            }
        }
    }

    pos
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(s: &str) -> Buffer {
        Buffer::from_str(s)
    }

    #[test]
    fn test_move_left_right() {
        let b = buf("hello");
        assert_eq!(Motion::Left.apply(Position::new(0, 2), &b), Position::new(0, 1));
        assert_eq!(Motion::Right.apply(Position::new(0, 2), &b), Position::new(0, 3));
    }

    #[test]
    fn test_word_forward() {
        let b = buf("hello world foo");
        let pos = word_forward(Position::new(0, 0), &b);
        assert_eq!(pos, Position::new(0, 6));
    }

    #[test]
    fn test_word_backward() {
        let b = buf("hello world foo");
        let pos = word_backward(Position::new(0, 6), &b);
        assert_eq!(pos, Position::new(0, 0));
    }

    #[test]
    fn test_match_bracket() {
        let b = buf("(hello (world))");
        assert_eq!(
            match_bracket(Position::new(0, 0), &b),
            Position::new(0, 14)
        );
        assert_eq!(
            match_bracket(Position::new(0, 7), &b),
            Position::new(0, 13)
        );
    }

    #[test]
    fn test_first_non_blank() {
        let b = buf("    hello");
        assert_eq!(first_non_blank(Position::new(0, 0), &b), Position::new(0, 4));
    }
}
