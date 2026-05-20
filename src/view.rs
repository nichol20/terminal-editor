use crate::terminal::{CursorMove, Terminal};
use std::io;

#[derive(Default)]
pub struct View;

impl View {
    pub fn render(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        for current_row in 0..terminal.size()?.height {
            terminal.move_cursor_to(CursorMove::Position {
                x: 0,
                y: current_row,
            })?;
            terminal.clear_line()?;
            terminal.print("~")?;
        }
        Ok(())
    }
}
