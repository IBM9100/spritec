use std::num::NonZeroU32;

use super::{Render, RenderNode, RenderLayout, LayoutType, Size};

#[derive(Debug)]
pub enum LayoutNode {
    Render(Render),
    Grid(GridLayout),
    /// An empty slot, used to create a gap/empty cell in the layout
    Empty {size: Size},
}

impl From<RenderNode> for LayoutNode {
    fn from(node: RenderNode) -> Self {
        use RenderNode::*;
        use LayoutType::*;
        match node {
            Render(render) => LayoutNode::Render(render),
            Layout(RenderLayout {nodes, layout: Grid {cols}}) => {
                let layout_nodes = nodes.into_iter().map(Into::into).collect();
                LayoutNode::Grid(GridLayout::new(layout_nodes, cols))
            },
            Empty {size} => LayoutNode::Empty {size},
        }
    }
}

impl LayoutNode {
    pub fn size(&self) -> Size {
        use LayoutNode::*;

        match self {
            Render(render) => render.size,
            Grid(grid) => grid.size(),
            Empty {size} => *size,
        }
    }

    pub fn iter_targets(self) -> LayoutTargetIter {
        LayoutTargetIter {
            node: Some(self),
            current: 0,
        }
    }
}

/// A fully-computed grid layout
#[derive(Debug)]
pub struct GridLayout {
    pub cells: Vec<LayoutNode>,
    pub cell_width: NonZeroU32,
    pub cell_height: NonZeroU32,
    pub rows: NonZeroU32,
    pub cols: NonZeroU32,
}

impl GridLayout {
    pub fn new(cells: Vec<LayoutNode>, cols: NonZeroU32) -> Self {
        assert!(!cells.is_empty(), "zero-cell grid layouts are not supported");

        let Size {width: cell_width, height: cell_height} = cells.iter().fold(
            Size::min_value(),
            |acc, cell| acc.max(cell.size()),
        );

        // ceiling division - https://stackoverflow.com/a/2745086/551904
        let rows = (cells.len() - 1) as u32 / cols.get() + 1;
        // Safe because cells.len() > 0 and we always add 1 in the end of the calculation of rows
        let rows = unsafe { NonZeroU32::new_unchecked(rows) };

        Self {cells, cell_width, cell_height, rows, cols}
    }

    /// Returns the total size of the image generated by this layout
    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    /// The total width of the image generated by this layout
    pub fn width(&self) -> NonZeroU32 {
        // Safe because multiplying two non-zero values cannot be zero
        unsafe { NonZeroU32::new_unchecked(self.cell_width.get() * self.cols.get()) }
    }

    /// The total height of the image generated by this layout
    pub fn height(&self) -> NonZeroU32 {
        // Safe because multiplying two non-zero values cannot be zero
        unsafe { NonZeroU32::new_unchecked(self.cell_height.get() * self.rows.get()) }
    }
}

/// The offset in the image to draw at
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutOffset {
    pub x: u32,
    pub y: u32,
}

/// Iterator over layout nodes and the target area they should be drawn into
pub struct LayoutTargetIter {
    node: Option<LayoutNode>,
    current: u32,
}

impl Iterator for LayoutTargetIter {
    type Item = (LayoutOffset, LayoutNode);

    fn next(&mut self) -> Option<Self::Item> {
        use LayoutNode::*;
        match self.node.take() {
            None => None,

            Some(node@Render(_)) | Some(node@Empty {..}) => {
                // Draw from the corner over the entire image
                let target = LayoutOffset {x: 0, y: 0};
                Some((target, node))
            },

            Some(Grid(grid)) => {
                let GridLayout {mut cells, cell_width, cell_height, rows, cols} = grid;

                // Stop once there are no more cells to yield
                if cells.is_empty() {
                    return None;
                }

                let current = self.current;

                let row = current / cols.get();
                let col = current % cols.get();
                let target = LayoutOffset {
                    x: col * cell_width.get(),
                    y: row * cell_height.get(),
                };

                self.current += 1;

                let node = cells.remove(0);
                // Reconstruct the node with the remaining cells
                self.node = Some(LayoutNode::Grid(GridLayout {
                    cells,
                    cell_width,
                    cell_height,
                    rows,
                    cols,
                }));

                Some((target, node))
            },
        }
    }
}
