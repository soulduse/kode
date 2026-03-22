use kode_core::geometry::Position;
use kode_editor::buffer::Buffer;

/// Text object types for vim `i`/`a` selections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObject {
    InnerWord,
    AroundWord,
    InnerParen,
    AroundParen,
    InnerBracket,
    AroundBracket,
    InnerBrace,
    AroundBrace,
    InnerQuote(char),
    AroundQuote(char),
}

/// Returns (start, end) positions for a text object.
impl TextObject {
    pub fn range(&self, pos: Position, buffer: &Buffer) -> Option<(Position, Position)> {
        match self {
            TextObject::InnerWord => inner_word(pos, buffer),
            TextObject::AroundWord => around_word(pos, buffer),
            TextObject::InnerParen => find_surrounding(pos, buffer, '(', ')', false),
            TextObject::AroundParen => find_surrounding(pos, buffer, '(', ')', true),
            TextObject::InnerBracket => find_surrounding(pos, buffer, '[', ']', false),
            TextObject::AroundBracket => find_surrounding(pos, buffer, '[', ']', true),
            TextObject::InnerBrace => find_surrounding(pos, buffer, '{', '}', false),
            TextObject::AroundBrace => find_surrounding(pos, buffer, '{', '}', true),
            TextObject::InnerQuote(q) => find_quotes(pos, buffer, *q, false),
            TextObject::AroundQuote(q) => find_quotes(pos, buffer, *q, true),
        }
    }
}

fn inner_word(pos: Position, buffer: &Buffer) -> Option<(Position, Position)> {
    let line_str = buffer.line_to_string(pos.line)?;
    let chars: Vec<char> = line_str.chars().collect();
    if pos.col >= chars.len() {
        return None;
    }

    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let start_is_word = is_word(chars[pos.col]);

    let mut start = pos.col;
    while start > 0 && is_word(chars[start - 1]) == start_is_word {
        start -= 1;
    }

    let mut end = pos.col;
    while end + 1 < chars.len() && is_word(chars[end + 1]) == start_is_word {
        end += 1;
    }

    Some((Position::new(pos.line, start), Position::new(pos.line, end)))
}

fn around_word(pos: Position, buffer: &Buffer) -> Option<(Position, Position)> {
    let (start, end) = inner_word(pos, buffer)?;
    let line_str = buffer.line_to_string(pos.line)?;
    let chars: Vec<char> = line_str.chars().collect();

    // Include trailing whitespace
    let mut new_end = end.col;
    while new_end + 1 < chars.len() && chars[new_end + 1].is_whitespace() {
        new_end += 1;
    }

    Some((start, Position::new(pos.line, new_end)))
}

fn find_surrounding(
    pos: Position,
    buffer: &Buffer,
    open: char,
    close: char,
    include_delimiters: bool,
) -> Option<(Position, Position)> {
    let line_str = buffer.line_to_string(pos.line)?;
    let chars: Vec<char> = line_str.chars().collect();

    // Find opening bracket going backward
    let mut depth = 0i32;
    let mut open_pos = None;
    for i in (0..=pos.col.min(chars.len().saturating_sub(1))).rev() {
        if chars[i] == close && i != pos.col {
            depth += 1;
        }
        if chars[i] == open {
            if depth == 0 {
                open_pos = Some(i);
                break;
            }
            depth -= 1;
        }
    }

    let open_col = open_pos?;

    // Find closing bracket going forward
    depth = 0;
    let mut close_pos = None;
    for i in open_col..chars.len() {
        if chars[i] == open {
            depth += 1;
        }
        if chars[i] == close {
            depth -= 1;
            if depth == 0 {
                close_pos = Some(i);
                break;
            }
        }
    }

    let close_col = close_pos?;

    if include_delimiters {
        Some((
            Position::new(pos.line, open_col),
            Position::new(pos.line, close_col),
        ))
    } else {
        Some((
            Position::new(pos.line, open_col + 1),
            Position::new(pos.line, close_col.saturating_sub(1)),
        ))
    }
}

fn find_quotes(
    pos: Position,
    buffer: &Buffer,
    quote: char,
    include_quotes: bool,
) -> Option<(Position, Position)> {
    let line_str = buffer.line_to_string(pos.line)?;
    let chars: Vec<char> = line_str.chars().collect();

    // Find the quote pair containing the cursor
    let mut start = None;
    let mut in_quotes = false;

    for (i, &c) in chars.iter().enumerate() {
        if c == quote {
            if !in_quotes {
                if i <= pos.col {
                    start = Some(i);
                    in_quotes = true;
                }
            } else if i >= pos.col {
                let s = start?;
                return if include_quotes {
                    Some((Position::new(pos.line, s), Position::new(pos.line, i)))
                } else {
                    Some((
                        Position::new(pos.line, s + 1),
                        Position::new(pos.line, i.saturating_sub(1)),
                    ))
                };
            } else {
                in_quotes = false;
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(s: &str) -> Buffer {
        Buffer::from_str(s)
    }

    #[test]
    fn test_inner_word() {
        let b = buf("hello world");
        let (start, end) = inner_word(Position::new(0, 7), &b).unwrap();
        assert_eq!(start, Position::new(0, 6));
        assert_eq!(end, Position::new(0, 10));
    }

    #[test]
    fn test_inner_paren() {
        let b = buf("foo(bar, baz)");
        let (start, end) = find_surrounding(Position::new(0, 5), &b, '(', ')', false).unwrap();
        assert_eq!(start, Position::new(0, 4));
        assert_eq!(end, Position::new(0, 11));
    }

    #[test]
    fn test_inner_quote() {
        let b = buf(r#"say "hello world" now"#);
        let (start, end) = find_quotes(Position::new(0, 8), &b, '"', false).unwrap();
        assert_eq!(start, Position::new(0, 5));
        assert_eq!(end, Position::new(0, 15));
    }
}
