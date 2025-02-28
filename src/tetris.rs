use std::time::{Duration, Instant};

use crate::{block::Block as TBlock, board::Board};
use anyhow::Result;
use rand::prelude::*;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::{border, Marker},
    text::Line,
    widgets::{
        canvas::{self, Canvas, Context},
        Block, Widget,
    },
    DefaultTerminal, Frame,
};

#[derive(Debug)]
pub struct Tetris {
    board: Board<Color>,
    scale: u16,
    score: i32,
    exit: bool,
    rng: ThreadRng,
}

impl Default for Tetris {
    fn default() -> Self {
        Self::new(10, 20, 2)
    }
}

impl Tetris {
    pub fn new(width: usize, height: usize, scale: u16) -> Self {
        Self {
            board: Board::new(width, height),
            scale,
            score: 0,
            exit: false,
            rng: rand::rng(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last_update = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(20))? {
                self.handle_events()?;
            }
            if last_update.elapsed() >= Duration::from_millis(800) {
                let _ = self.board.down();
                self.update_board();
                last_update = Instant::now();
            }
        }

        Ok(())
    }

    fn update_board(&mut self) {
        const COLORS: [Color; 6] = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
        ];

        if self
            .board
            .try_down()
            .or_else(|_| {
                self.score += self.board.clear_filled_rows() as i32;
                self.board.spawn(
                    TBlock::new(*TBlock::SHAPES.choose(&mut self.rng).unwrap()),
                    *COLORS.choose(&mut self.rng).unwrap(),
                )
            })
            .is_err()
        {
            self.exit()
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = Rect {
            x: 0,
            y: 0,
            width: self.board.width() as u16 * self.scale * 2 + 2,
            height: self.board.height() as u16 * self.scale + 2,
        };
        if area.intersection(frame.area()) != area {
            frame.render_widget("too small", frame.area());
        } else {
            frame.render_widget(self, area);
        }
    }

    fn fill_square(&self, ctx: &mut Context<'_>, x: usize, y: usize) {
        let color = self.board.get(x, y).unwrap_or(Color::Reset);
        let cx = x as f64;
        let cy = (self.board.height() - y - 1) as f64;
        let line_count = 2 * self.scale;
        for i in 0..line_count {
            ctx.draw(&canvas::Line {
                x1: cx + 1.0 / line_count as f64,
                y1: cy + i as f64 / line_count as f64,
                x2: cx + 1.0,
                y2: cy + i as f64 / line_count as f64,
                color,
            });
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Left => {
                        if self.board.left().is_ok() {
                            self.update_board();
                        }
                    }
                    KeyCode::Right => {
                        if self.board.right().is_ok() {
                            self.update_board();
                        }
                    }
                    KeyCode::Up => {
                        if self.board.rotate().is_ok() {
                            self.update_board();
                        }
                    }
                    KeyCode::Down => {
                        self.board.drop();
                        self.update_board();
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &Tetris {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" tetris ".bold());
        let title_bottom = if self.score > 0 {
            Line::from(vec![
                " score: ".into(),
                self.score.to_string().blue().bold(),
                " ".into(),
            ])
        } else {
            Line::from(vec![
                " press ".into(),
                "<Q>".blue().bold(),
                " to quit ".into(),
            ])
        };

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(title_bottom.centered())
            .border_set(border::THICK);

        Canvas::default()
            .block(block)
            .x_bounds([0.0, self.board.width() as f64])
            .y_bounds([0.0, self.board.height() as f64])
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                for x in 0..self.board.width() {
                    for y in 0..self.board.height() {
                        self.fill_square(ctx, x, y);
                    }
                }
            })
            .render(area, buf);
    }
}
