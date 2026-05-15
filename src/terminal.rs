use std::{
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

pub struct Terminal {}

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Terminal {
    pub fn initialize() -> io::Result<()> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?;
        Ok(())
    }

    pub fn terminate() -> io::Result<()> {
        disable_raw_mode()
    }

    pub fn clear_screen() -> io::Result<()> {
        Self::queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> io::Result<()> {
        Self::queue_command(Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(position: Position) -> io::Result<()> {
        let Position { x, y } = position;
        Self::queue_command(MoveTo(x, y))
    }

    pub fn hide_cursor() -> io::Result<()> {
        Self::queue_command(Hide)
    }

    pub fn show_cursor() -> io::Result<()> {
        Self::queue_command(Show)
    }

    pub fn print(content: impl Display) -> io::Result<()> {
        Self::queue_command(Print(content))
    }

    pub fn execute() -> io::Result<()> {
        stdout().flush()
    }

    pub fn size() -> io::Result<Size> {
        let (width, height) = size()?;
        Ok(Size { width, height })
    }

    fn queue_command<T: Command>(command: T) -> io::Result<()> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
