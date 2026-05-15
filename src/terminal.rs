use std::{
    fmt::Display,
    io::{self, Write, stdout},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};

pub struct Terminal {}

#[derive(Copy, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone)]
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
        queue!(stdout(), Clear(ClearType::All))
    }

    pub fn clear_line() -> io::Result<()> {
        queue!(stdout(), Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(position: Position) -> io::Result<()> {
        let Position { x, y } = position;
        queue!(stdout(), MoveTo(x, y))
    }

    pub fn hide_cursor() -> io::Result<()> {
        queue!(stdout(), Hide)
    }

    pub fn show_cursor() -> io::Result<()> {
        queue!(stdout(), Show)
    }

    pub fn print<T>(content: T) -> io::Result<()>
    where
        T: Display,
    {
        queue!(stdout(), Print(content))
    }

    pub fn execute() -> io::Result<()> {
        stdout().flush()
    }

    pub fn size() -> io::Result<Size> {
        let (width, height) = size()?;
        Ok(Size { width, height })
    }
}
