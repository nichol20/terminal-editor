use crossterm::event::Event;
use crossterm::event::{Event::Key, KeyCode::Char, KeyEvent, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::terminal::Terminal;

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { should_quit: false }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let height = Terminal::size()?.1;
        for current_row in 0..height {
            Terminal::move_cursor_to(0, current_row)?;
            Terminal::print("~")?;
        }
        Ok(())
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        loop {
            let event = read()?;
            self.evaluate_event(&event);
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
        }
        disable_raw_mode()?;
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            println!("Code: {code:?} Modifiers: {modifiers:?} \r");
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        if self.should_quit {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(0, 0)?;
        }
        Ok(())
    }
}
