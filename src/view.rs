use std::io;

use crate::{
    buffer::Buffer,
    line::Line,
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
            x: self.current_line_total_width_until_x() - self.scroll_offset.x,
            y: self.cursor_location.y - self.scroll_offset.y,
        }
    }

    pub fn current_line(&self) -> Option<&Line> {
        self.buffer.lines.get(self.cursor_location.y)
    }

    pub fn current_line_len(&self) -> usize {
        if let Some(l) = self.current_line() {
            return l.len();
        }
        0
    }

    pub fn current_line_total_width(&self) -> usize {
        if let Some(l) = self.current_line() {
            return l.total_width();
        }
        0
    }

    pub fn current_line_total_width_until_x(&self) -> usize {
        if let Some(l) = self.current_line() {
            return l.total_width_until(self.cursor_location.x);
        }
        0
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

    fn draw_debug_line(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        let cursor_position = self.get_position();
        let info_txt = format!(
            "cl {{x:{}, y:{}}}, so:{{x:{},y:{}}},cp{{x:{},y:{}}},tw:{},th:{},cltw:{},twux:{}",
            self.cursor_location.x,
            self.cursor_location.y,
            self.scroll_offset.x,
            self.scroll_offset.y,
            cursor_position.x,
            cursor_position.y,
            terminal.size()?.width,
            terminal.size()?.height,
            self.current_line_total_width(),
            self.current_line_total_width_until_x(),
        );
        terminal.move_cursor_to(Position {
            x: 0,
            y: terminal.size()?.height,
        });
        terminal.print(info_txt)?;
        Ok(())
    }

    pub fn render(&mut self, terminal: &mut Terminal) {
        if !self.needs_redraw {
            return;
        }

        let Size {
            width: terminal_width,
            height: terminal_height,
        } = terminal.size().unwrap_or_default();
        if terminal_width == 0 || terminal_height == 0 {
            return;
        }

        for current_row in 0..terminal_height {
            terminal.move_cursor_to(Position {
                x: 0,
                y: current_row,
            });

            let clear_line_result = terminal.clear_line();
            debug_assert!(clear_line_result.is_ok(), "Failed to clear line");

            let current_line_idx = current_row.saturating_add(self.scroll_offset.y);

            if let Some(line) = self.buffer.lines.get(current_line_idx) {
                let viewport_start = self.scroll_offset.x;
                let viewport_end = viewport_start.saturating_add(terminal_width);
                let mut source_column = 0_usize;
                let mut line_text = String::new();

                for text_frag in line.content.iter() {
                    let fragment_start = source_column;
                    let fragment_end =
                        fragment_start.saturating_add(text_frag.get_grapheme_width());
                    source_column = fragment_end;

                    // Completely left of the viewport
                    if fragment_end <= viewport_start {
                        continue;
                    }

                    // The viewport starts inside a wide grapheme. Preserve its remaining
                    // display cell as left angle bracket so subsequent graphemes stay aligned
                    if fragment_start < viewport_start {
                        let padding = fragment_end
                            .min(viewport_end)
                            .saturating_sub(viewport_start);

                        line_text.extend(std::iter::repeat('<').take(padding));

                        if fragment_end >= viewport_end {
                            break;
                        }
                        continue;
                    }

                    // The grapheme would cross the right edge
                    if fragment_end > viewport_end {
                        // display the cell as right angle bracket
                        // to indicate to the user that there is more text remaining
                        let padding = viewport_end.saturating_sub(fragment_start);
                        line_text.extend(std::iter::repeat('>').take(padding));
                        break;
                    }

                    if let Some(rep_c) = text_frag.replacement {
                        line_text.push(rep_c)
                    } else {
                        line_text.push_str(&text_frag.grapheme)
                    };
                }

                let print_line_result = terminal.print(line_text);
                debug_assert!(print_line_result.is_ok(), "Failed to print line");
                continue;
            }

            let til_result = terminal.print("~");
            debug_assert!(til_result.is_ok(), "Failed to to print '~'")
        }

        let debug_message_result = self.draw_debug_line(terminal);
        debug_assert!(debug_message_result.is_ok(), "Failed to draw debug message");

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
        let cur_line_len = self.current_line_len();
        let buffer_lines_len = self.buffer.lines.len();

        // ----- clamp X axis -----
        // allow place the cursor after the last character on the line (don't subtract 1)
        self.cursor_location.x = self.cursor_location.x.min(cur_line_len);

        let total_width_until_x = self.current_line_total_width_until_x();
        if total_width_until_x + 1 > self.scroll_offset.x + terminal_width {
            self.scroll_offset.x = total_width_until_x + 1 - terminal_width;
        }
        if self.scroll_offset.x > total_width_until_x {
            self.scroll_offset.x = total_width_until_x;
        }

        self.scroll_offset.x = self.scroll_offset.x.min(
            self.current_line_total_width()
                .saturating_sub(terminal_width)
                .saturating_add(1), // allow to show 1 char after the last
        );

        // ----- clamp Y axis -----
        // allow place the cursor below the last line (don't subtract 1)
        self.cursor_location.y = self.cursor_location.y.min(buffer_lines_len);
        self.scroll_offset.y = self.scroll_offset.y.min(
            buffer_lines_len
                .saturating_sub(terminal_height)
                .saturating_add(1), // allow to show 1 empty line below the last
        );
    }

    pub fn handle_action(&mut self, terminal: &mut Terminal, action: Action) -> () {
        let Size {
            width: terminal_width,
            height: terminal_height,
        } = terminal.size().unwrap_or_default();
        let cur_line_len = self.current_line_len();
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
