use kode_core::geometry::Position;

/// Cursor state supporting multi-cursor editing.
#[derive(Debug, Clone)]
pub struct CursorSet {
    cursors: Vec<Cursor>,
    primary: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub position: Position,
    /// Desired column when moving vertically through shorter lines.
    pub sticky_col: usize,
}

impl Cursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            position: Position::new(line, col),
            sticky_col: col,
        }
    }

    pub fn line(&self) -> usize {
        self.position.line
    }

    pub fn col(&self) -> usize {
        self.position.col
    }

    pub fn move_to(&mut self, line: usize, col: usize) {
        self.position = Position::new(line, col);
        self.sticky_col = col;
    }

    pub fn move_to_keeping_sticky(&mut self, line: usize, col: usize) {
        self.position = Position::new(line, col);
    }
}

impl CursorSet {
    pub fn new() -> Self {
        Self {
            cursors: vec![Cursor::new(0, 0)],
            primary: 0,
        }
    }

    pub fn primary(&self) -> &Cursor {
        &self.cursors[self.primary]
    }

    pub fn primary_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[self.primary]
    }

    pub fn all(&self) -> &[Cursor] {
        &self.cursors
    }

    pub fn add_cursor(&mut self, line: usize, col: usize) {
        self.cursors.push(Cursor::new(line, col));
    }

    pub fn clear_secondary(&mut self) {
        let primary = self.cursors[self.primary];
        self.cursors.clear();
        self.cursors.push(primary);
        self.primary = 0;
    }

    pub fn cursor_count(&self) -> usize {
        self.cursors.len()
    }
}

impl Default for CursorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_cursor() {
        let mut cs = CursorSet::new();
        assert_eq!(cs.primary().line(), 0);
        assert_eq!(cs.primary().col(), 0);

        cs.primary_mut().move_to(5, 10);
        assert_eq!(cs.primary().line(), 5);
        assert_eq!(cs.primary().col(), 10);
    }

    #[test]
    fn multi_cursor() {
        let mut cs = CursorSet::new();
        cs.add_cursor(3, 5);
        cs.add_cursor(7, 2);
        assert_eq!(cs.cursor_count(), 3);

        cs.clear_secondary();
        assert_eq!(cs.cursor_count(), 1);
    }
}
