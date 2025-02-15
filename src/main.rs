use anyhow::Result;
use tetris_rust::tetris::Tetris;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = Tetris::default().run(&mut terminal);
    ratatui::restore();
    result
}
