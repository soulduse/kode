use kode_core::geometry::Position;

/// A single selection defined by anchor and head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
}

impl Selection {
    pub fn new(anchor: Position, head: Position) -> Self {
        Self { anchor, head }
    }

    pub fn caret(pos: Position) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Returns (start, end) with start <= end.
    pub fn ordered(&self) -> (Position, Position) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.ordered();
        pos >= start && pos <= end
    }
}

/// Set of selections (for multi-cursor).
#[derive(Debug, Clone)]
pub struct SelectionSet {
    selections: Vec<Selection>,
    primary: usize,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            selections: vec![Selection::caret(Position::new(0, 0))],
            primary: 0,
        }
    }

    pub fn primary(&self) -> &Selection {
        &self.selections[self.primary]
    }

    pub fn primary_mut(&mut self) -> &mut Selection {
        &mut self.selections[self.primary]
    }

    pub fn all(&self) -> &[Selection] {
        &self.selections
    }

    pub fn set_primary(&mut self, selection: Selection) {
        self.selections[self.primary] = selection;
    }

    pub fn clear(&mut self) {
        self.selections = vec![Selection::caret(Position::new(0, 0))];
        self.primary = 0;
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_ordered() {
        let sel = Selection::new(Position::new(5, 3), Position::new(2, 1));
        let (start, end) = sel.ordered();
        assert_eq!(start, Position::new(2, 1));
        assert_eq!(end, Position::new(5, 3));
    }

    #[test]
    fn selection_empty() {
        let sel = Selection::caret(Position::new(1, 1));
        assert!(sel.is_empty());
    }
}
