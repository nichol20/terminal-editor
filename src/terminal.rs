use std::{
    cmp::{max, min},
    fmt::Display,
    io::{self, Write, stdout},
};

use crossterm::{
    Command,
    cursor::{Hide, MoveTo, Show},
    queue,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

#[derive(Default)]
pub struct Terminal {
    pub cursor_location: Location,
    pub queue_cursor_location: Location,
}

#[derive(Copy, Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Copy, Clone)]
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

#[allow(clippy::unused_self)]
impl Terminal {
    pub fn initialize(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        self.clear_screen()?;
        self.execute()?;
        Ok(())
    }

    pub fn terminate(&self) -> io::Result<()> {
        disable_raw_mode()
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        self.queue_command(Clear(ClearType::All))
    }

    pub fn clear_line(&self) -> io::Result<()> {
        self.queue_command(Clear(ClearType::CurrentLine))
    }

    /// Moves the cursor to the given Position.
    /// # Arguments
    /// * `Position` - the  `Position`to move the cursor to. Will be truncated to `u16::MAX` if bigger.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn move_cursor_to(&mut self, cursor_move: Direction) -> io::Result<()> {
        match cursor_move {
            Direction::Position { x, y } => {
                self.queue_command(MoveTo(x as u16, y as u16))?;
                self.set_queue_cursor_location(Location { x, y });
            }
            Direction::Up(n) => {
                let new_y = max(0, self.queue_cursor_location.y.saturating_sub(n));
                self.queue_command(MoveTo(self.queue_cursor_location.x as u16, new_y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: self.queue_cursor_location.x,
                    y: new_y,
                });
            }
            Direction::Down(n) => {
                let new_y = min(
                    self.size()?.height,
                    self.queue_cursor_location.y.saturating_add(n),
                );
                self.queue_command(MoveTo(self.queue_cursor_location.x as u16, new_y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: self.queue_cursor_location.x,
                    y: new_y,
                });
            }
            Direction::Left(n) => {
                let new_x = max(0, self.queue_cursor_location.x.saturating_sub(n));
                self.queue_command(MoveTo(new_x as u16, self.queue_cursor_location.y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: new_x,
                    y: self.queue_cursor_location.y,
                });
            }
            Direction::Right(n) => {
                let new_x = min(
                    self.size()?.width,
                    self.queue_cursor_location.x.saturating_add(n),
                );
                self.queue_command(MoveTo(new_x as u16, self.queue_cursor_location.y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: new_x,
                    y: self.queue_cursor_location.y,
                });
            }
            Direction::Top => {
                self.queue_command(MoveTo(self.queue_cursor_location.x as u16, 0))?;
                self.set_queue_cursor_location(Location {
                    x: self.queue_cursor_location.x,
                    y: 0,
                });
            }
            Direction::Bottom => {
                let height = self.size()?.height;
                self.queue_command(MoveTo(self.queue_cursor_location.x as u16, height as u16))?;
                self.set_queue_cursor_location(Location {
                    x: self.queue_cursor_location.x,
                    y: height,
                });
            }
            Direction::LineEnd => {
                let width = self.size()?.width;
                self.queue_command(MoveTo(width as u16, self.queue_cursor_location.y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: width,
                    y: self.queue_cursor_location.y,
                });
            }
            Direction::LineStart => {
                self.queue_command(MoveTo(0, self.queue_cursor_location.y as u16))?;
                self.set_queue_cursor_location(Location {
                    x: 0,
                    y: self.queue_cursor_location.y,
                });
            }
        }
        Ok(())
    }

    fn set_queue_cursor_location(&mut self, location: Location) {
        self.queue_cursor_location = location;
    }

    fn set_cursor_location(&mut self, location: Location) {
        self.cursor_location = location;
    }

    pub fn hide_cursor(&self) -> io::Result<()> {
        self.queue_command(Hide)
    }

    pub fn show_cursor(&self) -> io::Result<()> {
        self.queue_command(Show)
    }

    pub fn print(&self, content: impl Display) -> io::Result<()> {
        self.queue_command(Print(content))
    }

    pub fn execute(&mut self) -> io::Result<()> {
        self.set_cursor_location(self.queue_cursor_location);
        stdout().flush()
    }

    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize` < `z` < `u16`
    pub fn size(&self) -> io::Result<Size> {
        let (width_u16, height_u16) = size()?;
        #[allow(clippy::as_conversions)]
        Ok(Size {
            width: width_u16 as usize,
            height: height_u16 as usize,
        })
    }

    fn queue_command(&self, command: impl Command) -> io::Result<()> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
