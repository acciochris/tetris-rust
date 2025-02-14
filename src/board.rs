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

    fn update_block(&mut self, block: Block, value: T) -> Result<()> {
        // blog idea: double borrow, current_block immutable, board mutable
        // first clear current
        let current = self.current_block.as_ref().cloned();
        let mut orig_value = None;
        if let Some(orig) = current.as_ref() {
            // store original value in case of rollback; should be same across all coords
            let (x0, y0) = orig.coords()[0];
            orig_value = self.get(x0 as usize, y0 as usize).as_ref().cloned();

            for &(x, y) in orig.coords() {
                self.clear(x as usize, y as usize);
            }
        }

        // then replace with new block if valid
        let result = self.check_block(&block);
        if result.is_ok() {
            for &(x, y) in block.coords() {
                self.set(x as usize, y as usize, value.clone());
            }
            self.current_block = Some(block);
        } else if let Some(orig) = current.as_ref() {
            // roll back
            for &(x, y) in orig.coords() {
                self.set(
                    x as usize,
                    y as usize,
                    orig_value.as_ref().cloned().unwrap(),
                );
            }
        }

        result
    }

    pub fn spawn(&mut self, block: Block, value: T) -> Result<()> {
        // find topmost block and translate to center for spawning
        let (x, y) = *block.coords().iter().min_by_key(|(_, y)| *y).unwrap();

        // drop current block; required to prevent clearing current in update_block()
        self.current_block = None;

        self.update_block(block.translate((self.width / 2) as i32 - x, -y), value)?;

        Ok(())
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
    fn test_update_block() {
        let mut board = board! {
            0 0 0 0 0;
            0 0 0 0 0;
            0 0 0 0 0;
            0 1 0 1 1;
            1 1 1 0 1;
        };

        assert!(board.update_block(Block::new(Block::I), 2).is_ok());
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
        assert!(board
            .update_block(board.current_block.as_ref().unwrap().down(), 3)
            .is_ok());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                3 3 3 3 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
        assert!(board
            .update_block(board.current_block.as_ref().unwrap().rotate(), 4)
            .is_err());
        assert_eq!(
            board.board,
            board! {
                0 0 0 0 0;
                3 3 3 3 0;
                0 0 0 0 0;
                0 1 0 1 1;
                1 1 1 0 1;
            }
            .board
        );
        assert!(board
            .update_block(
                board
                    .current_block
                    .as_ref()
                    .unwrap()
                    .translate(0, -1)
                    .rotate_about((0, 0)),
                5
            )
            .is_ok());
        assert_eq!(
            board.board,
            board! {
                5 0 0 0 0;
                5 0 0 0 0;
                5 0 0 0 0;
                5 1 0 1 1;
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
}
