mod viewport;

use std::io;

use crate::{
    buffer::Buffer,
    command::Action,
    terminal::{Position, Size, Terminal},
    view::viewport::Viewport,
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    viewport: Viewport,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            viewport: Viewport::default(),
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
            x: self
                .viewport
                .current_line_total_width_until_cursor(&self.buffer)
                .saturating_sub(self.viewport.scroll_offset.x),
            y: self
                .viewport
                .cursor
                .y
                .saturating_sub(self.viewport.scroll_offset.y),
        }
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
            "cl {{x:{}, y:{}}}, so:{{x:{},y:{}}},cp{{x:{},y:{}}},tw:{},th:{},cltw:{},twuc:{}",
            self.viewport.cursor.x,
            self.viewport.cursor.y,
            self.viewport.scroll_offset.x,
            self.viewport.scroll_offset.y,
            cursor_position.x,
            cursor_position.y,
            terminal.size()?.width,
            terminal.size()?.height,
            self.viewport.current_line_total_width(&self.buffer),
            self.viewport
                .current_line_total_width_until_cursor(&self.buffer),
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

            let current_line_idx = current_row.saturating_add(self.viewport.scroll_offset.y);

            if let Some(line) = self.buffer.lines.get(current_line_idx) {
                let viewport_start = self.viewport.scroll_offset.x;
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

    pub fn handle_action(&mut self, terminal: &mut Terminal, action: Action) -> () {
        let Size {
            width: terminal_width,
            height: terminal_height,
        } = terminal.size().unwrap_or_default();
        self.viewport
            .handle_action(action, &self.buffer, terminal_width, terminal_height);
        self.set_redraw_flag(true);
    }
}
