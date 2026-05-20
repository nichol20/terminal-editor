use std::io;

use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyModifiers, read};

use crate::terminal::{CursorMove, Size, Terminal};
use crate::view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    view: View,
}

impl Editor {
    pub fn run(&mut self) {
        self.terminal.initialize().unwrap();
        let result = self.repl();
        self.terminal.terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> io::Result<()> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> io::Result<()> {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up => {
                    self.terminal.move_cursor_to(CursorMove::MoveUp(1))?;
                }
                KeyCode::Down => {
                    self.terminal.move_cursor_to(CursorMove::MoveDown(1))?;
                }
                KeyCode::Left => {
                    self.terminal.move_cursor_to(CursorMove::MoveLeft(1))?;
                }
                KeyCode::Right => {
                    self.terminal.move_cursor_to(CursorMove::MoveRight(1))?;
                }
                KeyCode::Home => {
                    self.terminal.move_cursor_to(CursorMove::MoveLineStart)?;
                }
                KeyCode::End => {
                    self.terminal.move_cursor_to(CursorMove::MoveLineEnd)?;
                }
                KeyCode::PageUp => {
                    self.terminal.move_cursor_to(CursorMove::MoveTop)?;
                }
                KeyCode::PageDown => {
                    self.terminal.move_cursor_to(CursorMove::MoveBottom)?;
                }
                _ => (),
            }
        }
        self.terminal.execute()?;
        Ok(())
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        self.terminal.hide_cursor()?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            self.terminal.print("Goodbye.\r\n")?;
        } else {
            self.render()?;
            self.draw_welcome_message()?;
            self.terminal.move_cursor_to(CursorMove::Position {
                x: self.terminal.cursor_location.x,
                y: self.terminal.cursor_location.y,
            })?;
        }
        self.terminal.show_cursor()?;
        self.terminal.execute()?;
        Ok(())
    }

    fn render(&mut self) -> io::Result<()> {
        self.view.render(&mut self.terminal)
    }

    fn draw_welcome_message(&mut self) -> io::Result<()> {
        let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
        let Size { width, height } = self.terminal.size()?;
        let offset_y = 2_usize;
        #[allow(clippy::integer_division)]
        self.terminal.move_cursor_to(CursorMove::Position {
            x: (width.saturating_sub(welcome_message.len())) / 2,
            y: height.saturating_sub(offset_y),
        })?;
        welcome_message.truncate(width.saturating_sub(1_usize));
        self.terminal.print(welcome_message)?;
        Ok(())
    }
}
