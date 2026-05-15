use std::io;

use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode::Char, KeyEvent, KeyModifiers, read};

use crate::terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub const fn default() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> io::Result<()> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn refresh_screen(&self) -> io::Result<()> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            Self::draw_rows()?;
            Self::draw_welcome_message()?;
            Terminal::move_cursor_to(Position { x: 0, y: 0 })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    fn draw_rows() -> io::Result<()> {
        for current_row in 0..Terminal::size()?.height {
            Terminal::move_cursor_to(Position {
                x: 0,
                y: current_row,
            })?;
            Terminal::clear_line()?;
            Terminal::print("~")?;
        }
        Ok(())
    }

    fn draw_welcome_message() -> io::Result<()> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let Size { width, height } = Terminal::size()?;
        let offset_y = 2_u16;
        Terminal::move_cursor_to(Position {
            x: (width - welcome_message.len() as u16) / 2,
            y: height - offset_y,
        })?;
        welcome_message.truncate(width as usize - 1_usize);
        Terminal::print(welcome_message)?;
        Ok(())
    }
}
