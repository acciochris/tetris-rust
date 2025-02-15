use crate::block::Block;
use anyhow::{bail, Result};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Board<T: Clone> {
    board: VecDeque<Vec<Option<T>>>,
    width: usize,
    height: usize,
    current_block: Option<Block>,
}

impl<T: Clone> Board<T> {
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
            current_block: None,
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

    fn check_block(&self, block: &Block) -> Result<()> {
        if !block.coords().iter().all(|&(x, y)| {
            x >= 0
                && y >= 0
                && (x as usize) < self.width
                && (y as usize) < self.height
                && self.get(x as usize, y as usize).is_none()
        }) {
            bail!("invalid block location");
        }

        Ok(())
    }

    fn update_block(&mut self, f: impl FnOnce(Block) -> Block) -> Result<()> {
        // blog idea: double borrow, current_block immutable, board mutable
        // first clear current
        let current = self.current_block.take().expect("current_block is None");

        // get value; should be same across all coords
        let (x0, y0) = current.coords()[0];
        let value = self.get(x0 as usize, y0 as usize).clone().unwrap();

        // clear current block
        for &(x, y) in current.coords() {
            self.clear(x as usize, y as usize);
        }

        // check validity of new block
        let new = f(current.clone());
        let result = self.check_block(&new);

        // either roll back or draw new block
        let block = match result {
            Ok(_) => new,
            Err(_) => current,
        };
        for &(x, y) in block.coords() {
            self.set(x as usize, y as usize, value.clone());
        }

        self.current_block = Some(block);
        result
    }

    fn set_block(&mut self, block: Block, value: T) -> Result<()> {
        if self.current_block.is_some() {
            panic!("current_block exists, call update_block instead");
        }

        self.check_block(&block)?;
        for &(x, y) in block.coords() {
            self.set(x as usize, y as usize, value.clone());
        }

        self.current_block = Some(block);
        Ok(())
    }

    pub fn spawn(&mut self, block: Block, value: T) -> Result<()> {
        // find topmost block and translate to center for spawning
        let (x, y) = *block.coords().iter().min_by_key(|(_, y)| *y).unwrap();

        self.current_block = None;
        self.set_block(block.translate((self.width / 2) as i32 - x, -y), value)?;

        Ok(())
    }

    pub fn left(&mut self) -> Result<()> {
        self.update_block(|b| b.left())
    }

    pub fn right(&mut self) -> Result<()> {
        self.update_block(|b| b.right())
    }

    pub fn down(&mut self) -> Result<()> {
        self.update_block(|b| b.down())
    }

    pub fn rotate(&mut self) -> Result<()> {
        self.update_block(|b| b.rotate())
    }

    pub fn drop(&mut self) {
        // FIXME: use binary search to optimize this
        while let Ok(_) = self.down() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs a Board from integers. For testing purposes only
    ///
    /// ```
    /// let b = board! {
    ///     0 0 0;
    ///     1 1 1;
    ///     2 2 2;
    /// };
    /// ```
    macro_rules! board {
        ($($($x:expr)+);+ $(;)?) => {
            {
                let board = VecDeque::from(vec![$(vec![$(match $x {
                    0 => None,
                    x => Some(x)
                }),+]),+]);
                let width = board[0].len();
                let height = board.len();
                Board { board, width, height, current_block: None }
            }
        };
    }

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

    #[test]
    fn test_board_macro() {
        let b = board! {
            0 2 0;
            1 1 1;
        };

        assert_eq!(b.width(), 3);
        assert_eq!(b.height(), 2);
        assert_eq!(b.get(0, 0), &None);
        assert_eq!(b.get(1, 0), &Some(2));
        assert_eq!(b.get(2, 0), &None);
        assert_eq!(b.get(0, 1), &Some(1));
        assert_eq!(b.get(1, 1), &Some(1));
        assert_eq!(b.get(2, 1), &Some(1));
    }

    #[test]
    fn test_check_block() {
        let board = board! {
            0 0 0;
            0 0 0;
            0 1 1;
            0 1 1;
        };

        assert!(board.check_block(&Block::new(Block::I)).is_err());
        assert!(board
            .check_block(&Block::new(Block::I).rotate_about((0, 0)))
            .is_ok());
        assert!(board
            .check_block(&Block::new(Block::I).rotate_about((0, 0)).down())
            .is_err());

        assert!(board.check_block(&Block::new(Block::O)).is_ok());
        assert!(board.check_block(&Block::new(Block::O).down()).is_err());
        assert!(board
            .check_block(&Block::new(Block::O).right().right())
            .is_err());
    }

    #[test]
    fn test_set_block() {
        let gen_board = || {
            board! {
                0 0 0 0 0;
                0 0 0 0 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
        };

        let mut board = gen_board();
        assert!(board.set_block(Block::new(Block::Z), 2).is_ok());
        assert_eq!(
            board.board,
            board! {
                2 2 0 0 0;
                0 2 2 0 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );

        let mut board = gen_board();
        assert!(board.set_block(Block::new(Block::L), 2).is_ok());
        assert_eq!(
            board.board,
            board! {
                2 0 0 0 0;
                2 0 0 0 0;
                2 2 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );

        let mut board = gen_board();
        assert!(board.set_block(Block::new(Block::L).down(), 2).is_err());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                0 0 0 0 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );

        let mut board = gen_board();
        assert!(board
            .set_block(Block::new(Block::I).translate(2, 0), 2)
            .is_err());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                0 0 0 0 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
    }

    #[test]
    fn test_update_block() {
        let mut board = board! {
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
            0 1 0 1 1;
            1 1 1 0 1;
        };

        assert!(board.set_block(Block::new(Block::I), 2).is_ok());
        assert_eq!(
            board.board,
            board! {
                2 2 2 2 0;
                0 0 0 0 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
        assert!(board.update_block(|b| b.down()).is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                2 2 2 2 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
        assert!(board.update_block(|b| b.rotate()).is_err());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                2 2 2 2 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
        assert!(board
            .update_block(|b| b.translate(0, -1).rotate_about((0, 0)))
            .is_ok());
        assert_eq!(
            board.board,
            board! {
                2 0 0 0 0;
                2 0 0 0 0;
                2 0 0 0 0;
                2 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
    }

    #[test]
    fn test_spawn() {
        let mut board = board! {
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
        };
        assert!(board.spawn(Block::new(Block::I), 1).is_ok());
        assert_eq!(
            board.board,
            board! {
                0 1 1 1 1;
                0 0 0 0 0;
                0 0 0 0 0;
                0 0 0 0 0;
            }
            .board
        );
        assert!(board.spawn(Block::new(Block::O), 2).is_err());

        let mut board2 = board! {
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
        };
        assert!(board2.spawn(Block::new(Block::J), 1).is_ok());
        assert_eq!(
            board2.board,
            board! {
                0 0 1 0 0;
                0 0 1 0 0;
                0 1 1 0 0;
                0 0 0 0 0;
            }
            .board
        );

        let mut board3 = board! {
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
        };
        assert!(board3.spawn(Block::new(Block::Z), 1).is_ok());
        assert_eq!(
            board3.board,
            board! {
                0 1 1 0 0;
                0 0 1 1 0;
                0 0 0 0 0;
                0 0 0 0 0;
            }
            .board
        );
    }

    #[test]
    fn test_actions() {
        let mut board = board! {
            0 0 0 0 0 0 0 0;
            0 0 0 0 0 0 0 0;
            0 0 0 0 0 0 0 0;
            0 0 0 0 0 0 0 0;
            0 0 0 0 0 0 0 0;
            0 1 0 0 0 0 0 0;
            0 1 0 1 1 1 1 1;
            1 1 1 0 1 1 1 1;
        };

        assert!(board.spawn(Block::new(Block::I), 2).is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 2 2 2 2 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 1 0 0 0 0 0 0;
                0 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.down().is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 0 0 0;
                0 0 0 2 2 2 2 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 1 0 0 0 0 0 0;
                0 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.rotate().is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 2 0 0 0;
                0 0 0 0 2 0 0 0;
                0 0 0 0 2 0 0 0;
                0 0 0 0 2 0 0 0;
                0 0 0 0 0 0 0 0;
                0 1 0 0 0 0 0 0;
                0 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        for _ in 0..4 {
            assert!(board.left().is_ok());
        }
        assert!(board.left().is_err());
        assert_eq!(
            board.board,
            board! {
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 1 0 0 0 0 0 0;
                0 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        board.drop();
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 1 0 0 0 0 0 0;
                2 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.spawn(Block::new(Block::Z), 3).is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 3 3 0 0 0;
                0 0 0 0 3 3 0 0;
                0 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 1 0 0 0 0 0 0;
                2 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.down().is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 0 0 0;
                0 0 0 3 3 0 0 0;
                0 0 0 0 3 3 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 1 0 0 0 0 0 0;
                2 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.rotate().is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 3 0 0 0;
                0 0 0 3 3 0 0 0;
                0 0 0 3 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 1 0 0 0 0 0 0;
                2 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.right().is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 3 0 0;
                0 0 0 0 3 3 0 0;
                0 0 0 0 3 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 1 0 0 0 0 0 0;
                2 1 0 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        assert!(board.left().is_ok());
        assert!(board.left().is_ok());
        board.drop();
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 3 0 0 0 0;
                2 1 3 3 0 0 0 0;
                2 1 3 1 1 1 1 1;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
        board.clear_filled_rows();
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                0 0 0 0 0 0 0 0;
                2 0 0 0 0 0 0 0;
                2 0 0 3 0 0 0 0;
                2 1 3 3 0 0 0 0;
                1 1 1 0 1 1 1 1;
            }
            .board
        );
    }
}
