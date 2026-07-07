use std::io;
use std::panic;
use std::path::Path;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};

use crate::terminal::{Direction, Terminal};
use crate::view::View;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    pub terminal: Terminal,
    view: View,
}

impl Editor {
    pub fn new() -> Self {
        let mut editor = Self {
            terminal: Terminal::default(),
            should_quit: false,
            view: View::default(),
        };

        // Set up a panic hook to ensure the terminal is properly terminated on panic
        let current_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            if let Err(err) = Terminal::default().terminate() {
                eprintln!("Failed to terminate terminal: {err:?}");
            }
            current_hook(panic_info);
        }));

        // Load the file specified in the command line arguments, if any
        let args: Vec<String> = std::env::args().collect();
        if let Some(first_arg) = args.get(1) {
            let path = Path::new(first_arg);

            if path.exists()
                && path.is_file()
                && let Some(path) = path.to_str()
            {
                editor.view.load(path).unwrap();
            }
        }

        editor.terminal.initialize().unwrap();

        editor
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => {
                    let result = self.evaluate_event(&event);
                    debug_assert!(result.is_ok(), "Failed to evaluate_event");
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event {err:?}");
                    }
                }
            }
        }
    }

    fn evaluate_event(&mut self, event: &Event) -> io::Result<()> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up => {
                    self.terminal.move_cursor_to(Direction::Up(1));
                }
                KeyCode::Down => {
                    self.terminal.move_cursor_to(Direction::Down(1));
                }
                KeyCode::Left => {
                    self.terminal.move_cursor_to(Direction::Left(1));
                }
                KeyCode::Right => {
                    self.terminal.move_cursor_to(Direction::Right(1));
                }
                KeyCode::Home => {
                    self.terminal.move_cursor_to(Direction::LineStart);
                }
                KeyCode::End => {
                    self.terminal.move_cursor_to(Direction::LineEnd);
                }
                KeyCode::PageUp => {
                    self.terminal.move_cursor_to(Direction::Top);
                }
                KeyCode::PageDown => {
                    self.terminal.move_cursor_to(Direction::Bottom);
                }
                _ => (),
            }
        }
        if let Event::Resize(_, _) = event {
            self.view.set_redraw_flag(true);
        }
        self.terminal.execute()?;
        Ok(())
    }

    fn refresh_screen(&mut self) {
        let _ = self.terminal.hide_cursor();
        
        self.view.render(&mut self.terminal);
        self.terminal.move_cursor_to(Direction::Position {
            x: self.terminal.cursor_location.x,
            y: self.terminal.cursor_location.y,
        });
        
        let _ = self.terminal.show_cursor();
        let _ = self.terminal.execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        if let Err(err) = self.terminal.terminate() {
            dbg!("Error dropping terminal: {:?}", err);
        }
        let _ = self.terminal.print("Goodbye.\r\n");
    }
}
