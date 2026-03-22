use kode_core::geometry::Rect;

use crate::pane::PaneId;

/// Split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Binary tree layout for pane arrangement.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LayoutNode {
    Leaf(PaneId),
    Split {
        direction: Direction,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    /// Compute screen rects for all leaf panes.
    pub fn compute_rects(&self, available: Rect) -> Vec<(PaneId, Rect)> {
        let mut result = Vec::new();
        self.compute_rects_inner(available, &mut result);
        result
    }

    fn compute_rects_inner(&self, rect: Rect, result: &mut Vec<(PaneId, Rect)>) {
        match self {
            LayoutNode::Leaf(id) => {
                result.push((*id, rect));
            }
            LayoutNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (r1, r2) = split_rect(rect, *direction, *ratio);
                first.compute_rects_inner(r1, result);
                second.compute_rects_inner(r2, result);
            }
        }
    }

    /// Split a leaf pane into two.
    pub fn split(&mut self, pane_id: PaneId, new_pane_id: PaneId, direction: Direction) {
        if let LayoutNode::Leaf(id) = self {
            if *id == pane_id {
                *self = LayoutNode::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(LayoutNode::Leaf(pane_id)),
                    second: Box::new(LayoutNode::Leaf(new_pane_id)),
                };
                return;
            }
        }
        if let LayoutNode::Split { first, second, .. } = self {
            first.split(pane_id, new_pane_id, direction);
            second.split(pane_id, new_pane_id, direction);
        }
    }

    /// Remove a pane from the layout. Returns true if found.
    pub fn remove(&mut self, pane_id: PaneId) -> bool {
        match self {
            LayoutNode::Leaf(id) => *id == pane_id,
            LayoutNode::Split { first, second, .. } => {
                if first.remove(pane_id) {
                    *self = *second.clone();
                    true
                } else if second.remove(pane_id) {
                    *self = *first.clone();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Find all leaf pane IDs.
    pub fn pane_ids(&self) -> Vec<PaneId> {
        match self {
            LayoutNode::Leaf(id) => vec![*id],
            LayoutNode::Split { first, second, .. } => {
                let mut ids = first.pane_ids();
                ids.extend(second.pane_ids());
                ids
            }
        }
    }
}

/// Find the adjacent pane in the given direction.
pub fn find_adjacent(
    pane_id: PaneId,
    direction: Direction,
    rects: &[(PaneId, Rect)],
) -> Option<PaneId> {
    let current = rects.iter().find(|(id, _)| *id == pane_id)?;
    let current_rect = &current.1;

    // Center of current pane
    let cx = current_rect.x() + current_rect.width() / 2.0;
    let cy = current_rect.y() + current_rect.height() / 2.0;

    let mut best: Option<(PaneId, f32)> = None;

    for &(id, ref rect) in rects {
        if id == pane_id {
            continue;
        }

        let rx = rect.x() + rect.width() / 2.0;
        let ry = rect.y() + rect.height() / 2.0;

        let is_candidate = match direction {
            Direction::Vertical => {
                // "Vertical split" direction — looking left/right
                // We'll use this for both left and right; caller decides sign
                true
            }
            Direction::Horizontal => true,
        };

        if !is_candidate {
            continue;
        }

        let dist = ((rx - cx).powi(2) + (ry - cy).powi(2)).sqrt();
        if best.is_none() || dist < best.unwrap().1 {
            best = Some((id, dist));
        }
    }

    best.map(|(id, _)| id)
}

/// Direction-aware adjacent pane lookup.
pub fn find_pane_in_direction(
    pane_id: PaneId,
    dir: FocusDirection,
    rects: &[(PaneId, Rect)],
) -> Option<PaneId> {
    let current = rects.iter().find(|(id, _)| *id == pane_id)?;
    let cr = &current.1;
    let cx = cr.x() + cr.width() / 2.0;
    let cy = cr.y() + cr.height() / 2.0;

    let mut best: Option<(PaneId, f32)> = None;

    for &(id, ref rect) in rects {
        if id == pane_id {
            continue;
        }

        let rx = rect.x() + rect.width() / 2.0;
        let ry = rect.y() + rect.height() / 2.0;

        let in_direction = match dir {
            FocusDirection::Left => rx < cx,
            FocusDirection::Right => rx > cx,
            FocusDirection::Up => ry < cy,
            FocusDirection::Down => ry > cy,
        };

        if !in_direction {
            continue;
        }

        let dist = ((rx - cx).powi(2) + (ry - cy).powi(2)).sqrt();
        if best.is_none() || dist < best.unwrap().1 {
            best = Some((id, dist));
        }
    }

    best.map(|(id, _)| id)
}

/// Focus direction for pane navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Find the pane at a given point.
pub fn find_pane_at(rects: &[(PaneId, Rect)], point: kode_core::geometry::Point) -> Option<PaneId> {
    rects
        .iter()
        .find(|(_, rect)| rect.contains(point))
        .map(|(id, _)| *id)
}

fn split_rect(rect: Rect, direction: Direction, ratio: f32) -> (Rect, Rect) {
    match direction {
        Direction::Vertical => {
            let w1 = rect.width() * ratio;
            let w2 = rect.width() - w1;
            (
                Rect::new(rect.x(), rect.y(), w1, rect.height()),
                Rect::new(rect.x() + w1, rect.y(), w2, rect.height()),
            )
        }
        Direction::Horizontal => {
            let h1 = rect.height() * ratio;
            let h2 = rect.height() - h1;
            (
                Rect::new(rect.x(), rect.y(), rect.width(), h1),
                Rect::new(rect.x(), rect.y() + h1, rect.width(), h2),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_single_pane() {
        let layout = LayoutNode::Leaf(0);
        let rects = layout.compute_rects(Rect::new(0.0, 0.0, 800.0, 600.0));
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].0, 0);
        assert!((rects[0].1.width() - 800.0).abs() < f32::EPSILON);
    }

    #[test]
    fn split_and_compute() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);

        let rects = layout.compute_rects(Rect::new(0.0, 0.0, 800.0, 600.0));
        assert_eq!(rects.len(), 2);
        assert!((rects[0].1.width() - 400.0).abs() < f32::EPSILON);
        assert!((rects[1].1.width() - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    fn remove_pane() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);
        layout.remove(1);

        let ids = layout.pane_ids();
        assert_eq!(ids, vec![0]);
    }
}
