use crate::{
    buffer::{Buffer, Line},
    command::{Action, Direction},
};

#[derive(Copy, Clone, Default)]
pub(super) struct Cursor {
    pub(super) x: usize,
    pub(super) y: usize,
}

#[derive(Copy, Clone, Default)]
pub(super) struct ScrollOffset {
    pub(super) x: usize,
    pub(super) y: usize,
}

#[derive(Default)]
pub(super) struct Viewport {
    pub(super) cursor: Cursor,
    pub(super) scroll_offset: ScrollOffset,
}

impl Viewport {
    fn current_line<'a>(&self, buffer: &'a Buffer) -> Option<&'a Line> {
        buffer.line(self.cursor.y)
    }

    fn current_line_len(&self, buffer: &Buffer) -> usize {
        self.current_line(buffer).map_or(0, Line::len)
    }

    pub(super) fn current_line_total_width(&self, buffer: &Buffer) -> usize {
        self.current_line(buffer).map_or(0, Line::total_width)
    }

    pub(super) fn current_line_total_width_until_cursor(&self, buffer: &Buffer) -> usize {
        self.current_line(buffer)
            .map_or(0, |line| line.total_width_until(self.cursor.x))
    }

    pub(super) fn handle_action(
        &mut self,
        action: Action,
        buffer: &Buffer,
        terminal_width: usize,
        terminal_height: usize,
    ) {
        let cur_line_len = self.current_line_len(buffer);
        let buf_line_count = buffer.line_count();

        match action {
            Action::Move(direction) => match direction {
                Direction::Position { x, y } => {
                    self.cursor.x = x;
                    self.cursor.y = y;
                }
                Direction::Up(n) => {
                    self.cursor.y = self.cursor.y.saturating_sub(n);
                    if self.scroll_offset.y > self.cursor.y {
                        self.scroll_offset.y = self.scroll_offset.y.saturating_sub(n);
                    }
                }
                Direction::Down(n) => {
                    self.cursor.y = self.cursor.y.saturating_add(n);
                    if self.cursor.y + 1 > self.scroll_offset.y + terminal_height {
                        self.scroll_offset.y += n;
                    }
                }
                Direction::Left(n) => {
                    if self.cursor.x == 0 && self.cursor.y > 0 {
                        self.cursor.y -= 1;
                        self.handle_action(
                            Action::Move(Direction::LineEnd),
                            buffer,
                            terminal_width,
                            terminal_height,
                        );
                        return;
                    }

                    self.cursor.x = self.cursor.x.saturating_sub(n);
                }
                Direction::Right(n) => {
                    self.cursor.x = self.cursor.x.saturating_add(n);

                    if self.cursor.x > cur_line_len && self.cursor.y < buf_line_count {
                        self.cursor.y += 1;
                        self.handle_action(
                            Action::Move(Direction::LineStart),
                            buffer,
                            terminal_width,
                            terminal_height,
                        );
                        return;
                    }
                }
                Direction::Top => {
                    self.scroll_offset.y = 0;
                    self.cursor.y = 0;
                }
                Direction::Bottom => {
                    self.cursor.y = buf_line_count;
                    self.scroll_offset.y = buf_line_count.saturating_sub(terminal_height);
                }
                Direction::PageUp => {
                    self.handle_action(
                        Action::Move(Direction::Up(terminal_height)),
                        buffer,
                        terminal_width,
                        terminal_height,
                    );
                    return;
                }
                Direction::PageDown => {
                    self.handle_action(
                        Action::Move(Direction::Down(terminal_height)),
                        buffer,
                        terminal_width,
                        terminal_height,
                    );
                    return;
                }
                Direction::LineEnd => {
                    self.cursor.x = cur_line_len;
                    self.scroll_offset.x = cur_line_len.saturating_sub(terminal_width);
                }
                Direction::LineStart => {
                    self.cursor.x = 0;
                    self.scroll_offset.x = 0;
                }
                _ => (),
            },
            Action::Resize => (),
        }

        self.clamp(buffer, terminal_width, terminal_height);
    }

    fn clamp(&mut self, buffer: &Buffer, terminal_width: usize, terminal_height: usize) {
        let cur_line_len = self.current_line_len(buffer);
        let buffer_line_count = buffer.line_count();

        // ----- clamp X axis -----
        // allow place the cursor after the last character on the line (don't subtract 1)
        self.cursor.x = self.cursor.x.min(cur_line_len);

        let total_width_until_cursor = self.current_line_total_width_until_cursor(buffer);
        if total_width_until_cursor + 1 > self.scroll_offset.x + terminal_width {
            self.scroll_offset.x = total_width_until_cursor + 1 - terminal_width;
        }
        if self.scroll_offset.x > total_width_until_cursor {
            self.scroll_offset.x = total_width_until_cursor;
        }

        self.scroll_offset.x = self.scroll_offset.x.min(
            self.current_line_total_width(buffer)
                .saturating_add(1) // allow to show 1 char after the last
                .saturating_sub(terminal_width),
        );

        // ----- clamp Y axis -----
        // allow place the cursor below the last line (don't subtract 1)
        self.cursor.y = self.cursor.y.min(buffer_line_count);

        if self.cursor.y + 1 > self.scroll_offset.y + terminal_height {
            self.scroll_offset.y = self.cursor.y + 1 - terminal_height;
        }
        if self.scroll_offset.y > self.cursor.y {
            self.scroll_offset.y = self.cursor.y;
        }

        self.scroll_offset.y = self.scroll_offset.y.min(
            buffer_line_count
                .saturating_add(1) // allow to show 1 empty line below the last
                .saturating_sub(terminal_height),
        );
    }
}
