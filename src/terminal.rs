use std::{
    cmp::{max, min},
    fmt::Display,
    io::{self, Write, stdout},
};

use crossterm::{
    Command,
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
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

#[derive(Copy, Clone, Default)]
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

impl Terminal {
    pub fn initialize(&mut self) -> io::Result<()> {
        self.enter_alternate_screen()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        self.execute()?;
        Ok(())
    }

    pub fn terminate(&mut self) -> io::Result<()> {
        let _ = self.clear_screen();
        self.leave_alternate_screen()?;
        self.show_cursor()?;
        self.execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn enter_alternate_screen(&self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)
    }

    #[allow(dead_code)]
    pub fn leave_alternate_screen(&self) -> io::Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)
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
    pub fn move_cursor_to(&mut self, direction: Direction) {
        let new_x;
        let new_y;
        let Size { width, height } = self.size().unwrap_or_default();

        match direction {
            Direction::Position { x, y } => {
                new_x = x;
                new_y = y;
            }
            Direction::Up(n) => {
                new_x = self.queue_cursor_location.x;
                new_y = max(0, self.queue_cursor_location.y.saturating_sub(n));
            }
            Direction::Down(n) => {
                new_x = self.queue_cursor_location.x;
                new_y = min(height, self.queue_cursor_location.y.saturating_add(n));
            }
            Direction::Left(n) => {
                new_x = max(0, self.queue_cursor_location.x.saturating_sub(n));
                new_y = self.queue_cursor_location.y;
            }
            Direction::Right(n) => {
                new_x = min(width, self.queue_cursor_location.x.saturating_add(n));
                new_y = self.queue_cursor_location.y;
            }
            Direction::Top => {
                new_x = self.queue_cursor_location.x;
                new_y = 0;
            }
            Direction::Bottom => {
                new_x = self.queue_cursor_location.x;
                new_y = height;
            }
            Direction::LineEnd => {
                new_x = width;
                new_y = self.queue_cursor_location.y;
            }
            Direction::LineStart => {
                new_x = 0;
                new_y = self.queue_cursor_location.y;
            }
        }

        let _ = self.queue_command(MoveTo(new_x as u16, new_y as u16));
        self.set_queue_cursor_location(Location { x: new_x, y: new_y });
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
