use std::{
    fmt::Display,
    io::{self, stdout},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};

pub struct Terminal {}

impl Terminal {
    pub fn initialize() -> io::Result<()> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(0, 0)?;
        Ok(())
    }

    pub fn terminate() -> io::Result<()> {
        disable_raw_mode()
    }

    pub fn clear_screen() -> io::Result<()> {
        execute!(stdout(), Clear(ClearType::All))
    }

    pub fn move_cursor_to(x: u16, y: u16) -> io::Result<()> {
        execute!(stdout(), MoveTo(x, y))
    }

    pub fn print<T>(content: T) -> io::Result<()>
    where
        T: Display,
    {
        execute!(stdout(), Print(content))
    }

    pub fn size() -> io::Result<(u16, u16)> {
        size()
    }
}
