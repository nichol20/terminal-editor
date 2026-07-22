use std::io;
use std::panic;
use std::path::Path;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};

use crate::command::Action;
use crate::command::Direction;
use crate::terminal::Terminal;
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
            let mut direction: Direction = Direction::None;
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up => direction = Direction::Up(1),
                KeyCode::Down => direction = Direction::Down(1),
                KeyCode::Left => direction = Direction::Left(1),
                KeyCode::Right => direction = Direction::Right(1),
                KeyCode::Home => direction = Direction::LineStart,
                KeyCode::End => direction = Direction::LineEnd,
                KeyCode::PageUp => direction = Direction::PageUp,
                KeyCode::PageDown => direction = Direction::PageDown,
                _ => (),
            }
            self.view
                .handle_action(&mut self.terminal, Action::Move(direction));
        }
        if let Event::Resize(_, _) = event {
            self.view.handle_action(&mut self.terminal, Action::Resize);
        }
        self.terminal.execute()?;
        Ok(())
    }

    fn refresh_screen(&mut self) {
        let _ = self.terminal.hide_cursor();

        self.view.render(&mut self.terminal);
        self.terminal.move_cursor_to(self.view.get_position());

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
