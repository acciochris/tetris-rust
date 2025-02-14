#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    coords: Vec<(i32, i32)>,
}

impl Block {
    /// Constructs a new block from slice.
    pub fn new(coords: &[(i32, i32)]) -> Self {
        Self {
            coords: coords.to_owned(),
        }
    }

    /// Getter for `coords`.
    fn coords(&self) -> &[(i32, i32)] {
        &self.coords
    }

    /// Returns a new block translated from the current by (dx, dy).
    pub fn translate(&self, dx: i32, dy: i32) -> Self {
        Self {
            coords: self.coords.iter().map(|(x, y)| (x + dx, y + dy)).collect(),
        }
    }

    /// Translate one unit to the left.
    pub fn left(&self) -> Self {
        self.translate(-1, 0)
    }

    /// Translate one unit to the right.
    pub fn right(&self) -> Self {
        self.translate(1, 0)
    }

    /// Translate one unit down.
    pub fn down(&self) -> Self {
        self.translate(0, 1)
    }

    /// Returns a new block rotated clockwise by 90 degrees about the center of the block.
    pub fn rotate(&self) -> Self {
        self.rotate_about(self.coords[0])
    }

    /// Returns a new block rotated clockwise by 90 degrees about `center`.
    fn rotate_about(&self, center: (i32, i32)) -> Self {
        let (x0, y0) = center;
        Self {
            coords: self
                .coords
                .iter()
                .map(|(x, y)| (x0 + y0 - y, -x0 + y0 + x))
                .collect(),
        }
    }

    pub fn is_valid(&self, pred: impl Fn((i32, i32)) -> bool) -> bool {
        self.coords.iter().copied().all(pred)
    }
}

#[cfg(test)]
mod tests {
    use super::Block;

    #[test]
    fn test_block_translate() {
        // horizontal strip
        let block = Block::new(&[(0, 0), (1, 0), (2, 0), (3, 0)]);

        assert_eq!(
            block.translate(-1, 1).coords(),
            &[(-1, 1), (0, 1), (1, 1), (2, 1)]
        );
        assert_eq!(block.left(), block.translate(-1, 0));
        assert_eq!(block.right(), block.translate(1, 0));
        assert_eq!(block.down(), block.translate(0, 1));
        assert_eq!(
            block.left().right().right().right().down().down().down(),
            block.translate(2, 3)
        );
    }

    #[test]
    fn test_block_rotate() {
        // horizontal strip
        let block = Block::new(&[(0, 0), (1, 0), (2, 0), (3, 0)]);

        assert_eq!(
            block.rotate_about((0, 0)).coords(),
            &[(0, 0), (0, 1), (0, 2), (0, 3)]
        );
        assert_eq!(
            block.rotate_about((3, 0)).coords(),
            &[(3, -3), (3, -2), (3, -1), (3, 0)]
        );
        assert_eq!(block.rotate().coords(), &[(0, 0), (0, 1), (0, 2), (0, 3)]);
        assert_eq!(block.rotate().rotate().rotate().rotate(), block);
    }
}
