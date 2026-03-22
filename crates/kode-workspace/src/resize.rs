use crate::layout::{Direction, LayoutNode};
use crate::pane::PaneId;

/// Minimum pane dimensions in pixels.
pub const MIN_PANE_WIDTH: f32 = 80.0;
pub const MIN_PANE_HEIGHT: f32 = 40.0;

/// Resize a pane by adjusting the split ratio of its parent.
/// `delta` is a ratio change (e.g., 0.05 to grow, -0.05 to shrink).
pub fn resize_pane(layout: &mut LayoutNode, pane_id: PaneId, delta: f32) -> bool {
    match layout {
        LayoutNode::Leaf(_) => false,
        LayoutNode::Split {
            ratio,
            first,
            second,
            ..
        } => {
            let first_ids = first.pane_ids();
            let second_ids = second.pane_ids();

            if first_ids.contains(&pane_id) {
                // Try to resize within the first child first
                if resize_pane(first, pane_id, delta) {
                    return true;
                }
                // Pane is a direct child of this split — adjust ratio
                let new_ratio = (*ratio + delta).clamp(0.1, 0.9);
                *ratio = new_ratio;
                true
            } else if second_ids.contains(&pane_id) {
                // Try to resize within the second child first
                if resize_pane(second, pane_id, delta) {
                    return true;
                }
                // Adjust ratio (shrink first to grow second)
                let new_ratio = (*ratio - delta).clamp(0.1, 0.9);
                *ratio = new_ratio;
                true
            } else {
                false
            }
        }
    }
}

/// Set all splits to equal ratios.
pub fn equalize_panes(layout: &mut LayoutNode) {
    if let LayoutNode::Split {
        ratio,
        first,
        second,
        ..
    } = layout
    {
        *ratio = 0.5;
        equalize_panes(first);
        equalize_panes(second);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Direction;

    #[test]
    fn resize_grow() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);

        assert!(resize_pane(&mut layout, 0, 0.1));

        if let LayoutNode::Split { ratio, .. } = &layout {
            assert!((*ratio - 0.6).abs() < f32::EPSILON);
        } else {
            panic!("Expected split");
        }
    }

    #[test]
    fn resize_clamp() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);

        // Try to grow beyond max
        resize_pane(&mut layout, 0, 0.9);
        if let LayoutNode::Split { ratio, .. } = &layout {
            assert!((*ratio - 0.9).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn equalize() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);
        resize_pane(&mut layout, 0, 0.2);
        equalize_panes(&mut layout);

        if let LayoutNode::Split { ratio, .. } = &layout {
            assert!((*ratio - 0.5).abs() < f32::EPSILON);
        }
    }
}
