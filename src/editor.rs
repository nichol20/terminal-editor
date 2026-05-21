use std::io;
use std::path::Path;

use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyModifiers, read};

use crate::terminal::{Direction, Terminal};
use crate::view::View;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    view: View,
}

impl Editor {
    pub fn run(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        if let Some(first_arg) = args.get(1) {
            let path = Path::new(first_arg);

            if path.exists() && path.is_file() {
                let path = path.to_str().unwrap();
                self.view.load(path).unwrap();
            }
        }

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
                    self.terminal.move_cursor_to(Direction::Up(1))?;
                }
                KeyCode::Down => {
                    self.terminal.move_cursor_to(Direction::Down(1))?;
                }
                KeyCode::Left => {
                    self.terminal.move_cursor_to(Direction::Left(1))?;
                }
                KeyCode::Right => {
                    self.terminal.move_cursor_to(Direction::Right(1))?;
                }
                KeyCode::Home => {
                    self.terminal.move_cursor_to(Direction::LineStart)?;
                }
                KeyCode::End => {
                    self.terminal.move_cursor_to(Direction::LineEnd)?;
                }
                KeyCode::PageUp => {
                    self.terminal.move_cursor_to(Direction::Top)?;
                }
                KeyCode::PageDown => {
                    self.terminal.move_cursor_to(Direction::Bottom)?;
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
            self.view.render(&mut self.terminal)?;
            self.terminal.move_cursor_to(Direction::Position {
                x: self.terminal.cursor_location.x,
                y: self.terminal.cursor_location.y,
            })?;
        }
        self.terminal.show_cursor()?;
        self.terminal.execute()?;
        Ok(())
    }
}
