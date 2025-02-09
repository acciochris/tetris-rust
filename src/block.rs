pub struct Block {
    coords: Vec<(i32, i32)>,
}

impl Block {
    pub fn new(coords: &[(i32, i32)]) -> Self {
        Self {
            coords: coords.to_owned(),
        }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> Self {
        Self {
            coords: self.coords.iter().map(|(x, y)| (x + dx, y + dy)).collect(),
        }
    }

    pub fn left(&self) -> Self {
        self.translate(-1, 0)
    }

    pub fn right(&self) -> Self {
        self.translate(1, 0)
    }

    pub fn down(&self) -> Self {
        self.translate(0, 1)
    }

    pub fn rotate_cw(&self) -> Self {
        // find a middle point as pivot
        let (x0, y0) = self.coords[self.coords.len() / 2];
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
