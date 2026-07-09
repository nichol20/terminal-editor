use std::{
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

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Default)]
pub struct Terminal {}

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
    pub fn move_cursor_to(&mut self, position: Position) {
        let _ = self.queue_command(MoveTo(position.x as u16, position.y as u16));
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
