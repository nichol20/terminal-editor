use std::io;

use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyModifiers, read};

use crate::terminal::{CursorMove, Terminal};
use crate::view::View;

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
}
