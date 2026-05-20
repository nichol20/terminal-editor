use crate::{
    buffer::Buffer,
    terminal::{CursorMove, Size, Terminal},
};
use std::io;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn render(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        for current_row in 0..terminal.size()?.height {
            terminal.move_cursor_to(CursorMove::Position {
                x: 0,
                y: current_row,
            })?;
            terminal.clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row) {
                terminal.print(line)?;
                continue;
            }
            terminal.print("~")?;
        }

        self.draw_welcome_message(terminal)?;
        Ok(())
    }

    pub fn draw_welcome_message(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let Size { width, height } = terminal.size()?;
        let offset_y = 2_usize;
        #[allow(clippy::integer_division)]
        terminal.move_cursor_to(CursorMove::Position {
            x: (width.saturating_sub(welcome_message.len())) / 2,
            y: height.saturating_sub(offset_y),
        })?;
        welcome_message.truncate(width.saturating_sub(1_usize));
        terminal.print(welcome_message)?;
        Ok(())
    }
}
