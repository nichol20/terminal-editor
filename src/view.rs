use std::io;

use crate::{
    buffer::{Buffer, Line},
    terminal::{Location, Position, Size, Terminal},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(dead_code)]
pub enum Direction {
    Position { x: usize, y: usize },
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Top,
    Bottom,
    PageUp,
    PageDown,
    LineEnd,
    LineStart,
    None,
}

pub enum Action {
    Move(Direction),
    Resize,
}

pub struct ScrollOffset {
    pub x: usize,
    pub y: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    pub cursor_location: Location,
    pub scroll_offset: ScrollOffset,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            cursor_location: Location { x: 0, y: 0 },
            scroll_offset: ScrollOffset { x: 0, y: 0 },
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

    pub fn get_position(&self) -> Position {
        Position {
            x: self.cursor_location.x - self.scroll_offset.x,
            y: self.cursor_location.y - self.scroll_offset.y,
        }
    }

    pub fn current_line(&self) -> Option<&Line> {
        self.buffer.lines.get(self.cursor_location.y)
    }

    fn draw_welcome_message(&mut self, terminal: &mut Terminal) -> io::Result<()> {
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

            let current_line_idx = current_row.saturating_add(self.scroll_offset.y);
            if let Some(line) = self.buffer.lines.get(current_line_idx) {
                let truncated_line = line.get(self.scroll_offset.x..);
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

    fn clamp_cursor(&mut self, terminal_width: usize, terminal_height: usize) {
        let cur_line_len = self
            .current_line()
            .unwrap_or(&Line {
                text: String::new(),
            })
            .len();
        let buffer_lines_len = self.buffer.lines.len();

        // allow place the cursor after the last character on the line (don't subtract 1)
        self.cursor_location.x = self.cursor_location.x.min(cur_line_len);
        // allow place the cursor below the last line (don't subtract 1)
        self.cursor_location.y = self.cursor_location.y.min(buffer_lines_len);
        self.scroll_offset.y = self.scroll_offset.y.min(
            buffer_lines_len
                .saturating_sub(terminal_height)
                .saturating_add(1), // allow to show 1 empty line below the last
        );
        self.scroll_offset.x = self.scroll_offset.x.min(
            cur_line_len
                .saturating_sub(terminal_width)
                .saturating_add(1), // allow to show 1 char after the last
        );
    }

    pub fn handle_action(&mut self, terminal: &mut Terminal, action: Action) -> () {
        let Size {
            width: terminal_width,
            height: terminal_height,
        } = terminal.size().unwrap_or_default();
        let cur_line_len = self
            .current_line()
            .unwrap_or(&Line {
                text: String::new(),
            })
            .len();
        let buf_lines_len = self.buffer.lines.len();

        match action {
            Action::Move(direction) => match direction {
                Direction::Position { x, y } => {
                    self.cursor_location.x = x;
                    self.cursor_location.y = y;
                }
                Direction::Up(n) => {
                    self.cursor_location.y = self.cursor_location.y.saturating_sub(n);
                    if self.scroll_offset.y > self.cursor_location.y {
                        self.scroll_offset.y = self.scroll_offset.y.saturating_sub(n);
                    }
                }
                Direction::Down(n) => {
                    self.cursor_location.y = self.cursor_location.y.saturating_add(n);
                    if self.cursor_location.y + 1 > self.scroll_offset.y + terminal_height {
                        self.scroll_offset.y += n;
                    }
                }
                Direction::Left(n) => {
                    if self.cursor_location.x == 0 && self.cursor_location.y > 0 {
                        self.cursor_location.y -= 1;
                        self.handle_action(terminal, Action::Move(Direction::LineEnd));
                        return;
                    }

                    self.cursor_location.x = self.cursor_location.x.saturating_sub(n);

                    if self.scroll_offset.x > self.cursor_location.x {
                        self.scroll_offset.x = self.scroll_offset.x.saturating_sub(n);
                    }
                }
                Direction::Right(n) => {
                    self.cursor_location.x = self.cursor_location.x.saturating_add(n);

                    if self.cursor_location.x > cur_line_len
                        && self.cursor_location.y < buf_lines_len
                    {
                        self.cursor_location.y += 1;
                        self.handle_action(terminal, Action::Move(Direction::LineStart));
                        return;
                    }

                    if self.cursor_location.x + 1 > self.scroll_offset.x + terminal_width {
                        self.scroll_offset.x += n;
                    }
                }
                Direction::Top => {
                    self.scroll_offset.y = 0;
                    self.cursor_location.y = 0;
                }
                Direction::Bottom => {
                    self.cursor_location.y = buf_lines_len;
                    self.scroll_offset.y = buf_lines_len.saturating_sub(terminal_height);
                }
                Direction::PageUp => {
                    self.handle_action(terminal, Action::Move(Direction::Up(terminal_height)));
                    return;
                }
                Direction::PageDown => {
                    self.handle_action(terminal, Action::Move(Direction::Down(terminal_height)));
                    return;
                }
                Direction::LineEnd => {
                    self.cursor_location.x = cur_line_len;
                    self.scroll_offset.x = cur_line_len.saturating_sub(terminal_width);
                }
                Direction::LineStart => {
                    self.cursor_location.x = 0;
                    self.scroll_offset.x = 0;
                }
                _ => (),
            },
            Action::Resize => (),
        }

        self.clamp_cursor(terminal_width, terminal_height);
        self.set_redraw_flag(true);
    }
}
