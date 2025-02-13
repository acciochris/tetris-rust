use std::collections::VecDeque;

#[derive(Debug)]
pub struct Board<T> {
    board: VecDeque<Vec<Option<T>>>,
    width: usize,
    height: usize,
}

impl<T> Board<T> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut board = VecDeque::new();
        board.resize_with(height, || {
            let mut row = Vec::new();
            row.resize_with(width, || None);
            row
        });
        Self {
            board,
            width,
            height,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get(&self, x: usize, y: usize) -> &Option<T> {
        &self.board[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.board[y][x] = Some(value);
    }

    pub fn clear(&mut self, x: usize, y: usize) {
        self.board[y][x] = None;
    }

    pub fn clear_filled_rows(&mut self) {
        self.board.retain(|row| row.iter().any(|x| x.is_none()));

        // insert new empty rows to maintain height
        for _ in 0..(self.height - self.board.len()) {
            let mut row = Vec::new();
            row.resize_with(self.width, || None);
            self.board.push_front(row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new() {
        let board = Board::<()>::new(4, 8);
        for x in 0..4 {
            for y in 0..8 {
                assert_eq!(board.get(x, y), &None);
            }
        }
    }

    #[test]
    fn test_board_clear_rows() {
        let mut board = Board::<()>::new(4, 8);

        // row 5 and 7 is full, row 6 is not
        board.set(0, 5, ());
        board.set(1, 5, ());
        board.set(2, 5, ());
        board.set(3, 5, ());
        board.set(0, 6, ());
        board.set(1, 6, ());
        board.set(3, 6, ());
        board.set(0, 7, ());
        board.set(1, 7, ());
        board.set(2, 7, ());
        board.set(3, 7, ());

        board.clear_filled_rows();

        for x in 0..4 {
            for y in 0..7 {
                assert_eq!(board.get(x, y), &None);
            }
        }
        assert_eq!(board.get(0, 7), &Some(()));
        assert_eq!(board.get(1, 7), &Some(()));
        assert_eq!(board.get(2, 7), &None);
        assert_eq!(board.get(3, 7), &Some(()));
    }
}
