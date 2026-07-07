use crate::{
    buffer::Buffer,
    terminal::{Direction, Size, Terminal},
};
use std::io;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
        }
    }
}

impl View {
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.buffer.load(path)?;
        self.set_redraw_flag(true);
        Ok(())
    }

    pub fn set_redraw_flag(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    pub fn render(&mut self, terminal: &mut Terminal) {
        if !self.needs_redraw {
            return;
        }

        let Size { width, height } = terminal.size().unwrap_or_default();
        if width == 0 || height == 0 {
            return;
        }

        for current_row in 0..height {
            terminal.move_cursor_to(Direction::Position {
                x: 0,
                y: current_row,
            });

            let clear_line_result = terminal.clear_line();
            debug_assert!(clear_line_result.is_ok(), "Failed to clear line");

            if let Some(line) = self.buffer.lines.get(current_row) {
                // Truncate the line to fit within the terminal width
                let truncated_line = line.get(0..width).unwrap_or(line);
                let print_line_result = terminal.print(truncated_line);
                debug_assert!(print_line_result.is_ok(), "Failed to print line");
                continue;
            }

            let til_result = terminal.print("~");
            debug_assert!(til_result.is_ok(), "Failed to to print '~'")
        }

        if self.buffer.is_empty() {
            let welcome_message_result = self.draw_welcome_message(terminal);
            debug_assert!(
                welcome_message_result.is_ok(),
                "Failed to draw welcome message"
            );
        }
        self.set_redraw_flag(false);
    }

    pub fn draw_welcome_message(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let Size { width, height } = terminal.size()?;
        let offset_y = 2_usize;

        #[allow(clippy::integer_division)]
        terminal.move_cursor_to(Direction::Position {
            x: (width.saturating_sub(welcome_message.len())) / 2,
            y: height.saturating_sub(offset_y),
        });
        // subtract 1 from the width to make room for the tilde
        welcome_message.truncate(width.saturating_sub(1_usize));
        terminal.print(welcome_message)?;
        Ok(())
    }
}
