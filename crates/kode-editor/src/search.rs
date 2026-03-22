use kode_core::geometry::Position;

use crate::buffer::Buffer;

/// Search match in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub start: Position,
    pub end: Position,
    pub char_range: std::ops::Range<usize>,
}

/// Find all occurrences of a pattern in the buffer.
pub fn find_all(buffer: &Buffer, pattern: &str, case_sensitive: bool) -> Vec<SearchMatch> {
    if pattern.is_empty() {
        return Vec::new();
    }

    let text = buffer.text();
    let (search_text, search_pattern);

    if case_sensitive {
        search_text = text.clone();
        search_pattern = pattern.to_string();
    } else {
        search_text = text.to_lowercase();
        search_pattern = pattern.to_lowercase();
    };

    let mut matches = Vec::new();
    let mut start = 0;

    while let Some(byte_pos) = search_text[start..].find(&search_pattern) {
        let absolute_byte = start + byte_pos;
        let char_start = text[..absolute_byte].chars().count();
        let char_end = char_start + pattern.chars().count();

        let start_line = buffer.char_to_line(char_start);
        let start_col = char_start - buffer.line_to_char(start_line);
        let end_line = buffer.char_to_line(char_end);
        let end_col = char_end - buffer.line_to_char(end_line);

        matches.push(SearchMatch {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
            char_range: char_start..char_end,
        });

        start = absolute_byte + search_pattern.len();
    }

    matches
}

/// Find the next match after a given position.
pub fn find_next(
    buffer: &Buffer,
    pattern: &str,
    after: Position,
    case_sensitive: bool,
) -> Option<SearchMatch> {
    let matches = find_all(buffer, pattern, case_sensitive);
    matches
        .into_iter()
        .find(|m| m.start > after)
        .or_else(|| {
            // Wrap around
            let matches = find_all(buffer, pattern, case_sensitive);
            matches.into_iter().next()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_all_basic() {
        let buf = Buffer::from_str("hello world hello");
        let matches = find_all(&buf, "hello", true);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].start, Position::new(0, 0));
        assert_eq!(matches[1].start, Position::new(0, 12));
    }

    #[test]
    fn find_case_insensitive() {
        let buf = Buffer::from_str("Hello HELLO hello");
        let matches = find_all(&buf, "hello", false);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn find_empty_pattern() {
        let buf = Buffer::from_str("hello");
        let matches = find_all(&buf, "", true);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_next_wrap() {
        let buf = Buffer::from_str("abc abc abc");
        let m = find_next(&buf, "abc", Position::new(0, 8), true);
        assert!(m.is_some());
        assert_eq!(m.unwrap().start, Position::new(0, 0));
    }
}
