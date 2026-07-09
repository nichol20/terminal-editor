use std::io;

use crate::{
    buffer::Buffer,
    terminal::{Location, Position, Size, Terminal},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum Direction {
    Position { x: usize, y: usize },
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Top,
    Bottom,
    LineEnd,
    LineStart,
}

pub enum Action {
    Move(Direction),
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    pub cursor_location: Location,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            cursor_location: Location { x: 0, y: 0 },
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
            terminal.move_cursor_to(Position {
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

    pub fn handle_action(&mut self, action: Action) -> () {
        match action {
            Action::Move(direction) => match direction {
                Direction::Position { x, y } => {
                    self.cursor_location.x = x;
                    self.cursor_location.y = y;
                }
                Direction::Up(n) => {
                    self.cursor_location.y = self.cursor_location.y.saturating_sub(n).max(0);
                }
                Direction::Down(n) => {
                    self.cursor_location.y = self
                        .cursor_location
                        .y
                        .saturating_add(n)
                        .min(self.buffer.lines.len());
                }
                Direction::Left(n) => {
                    self.cursor_location.x = self.cursor_location.x.saturating_sub(n).max(0);
                }
                Direction::Right(n) => {
                    self.cursor_location.x = self.cursor_location.x.saturating_add(n);
                }
                Direction::Top => {
                    self.cursor_location.y = 0;
                }
                Direction::Bottom => {
                    self.cursor_location.y = self.buffer.lines.len();
                }
                Direction::LineEnd => {
                    self.cursor_location.x = self.current_line().len();
                }
                Direction::LineStart => {
                    self.cursor_location.x = 0;
                }
            },
        }

        self.clamp_cursor();
    }

    pub fn clamp_cursor(&mut self) {
        let cur_line_len = self.current_line().len();
        self.cursor_location.x = self.cursor_location.x.min(cur_line_len);
    }

    pub fn current_line(&self) -> &str {
        self.buffer
            .lines
            .get(self.cursor_location.y)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn draw_welcome_message(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let Size { width, height } = terminal.size()?;
        let offset_y = 2_usize;

        #[allow(clippy::integer_division)]
        terminal.move_cursor_to(Position {
            x: (width.saturating_sub(welcome_message.len())) / 2,
            y: height.saturating_sub(offset_y),
        });
        // subtract 1 from width to let a space for tilde
        welcome_message.truncate(width.saturating_sub(1_usize));
        terminal.print(welcome_message)?;
        Ok(())
    }
}
