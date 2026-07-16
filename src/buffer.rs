use std::{fs::read_to_string, io};

use crate::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.lines = read_to_string(path)?.lines().map(Line::from).collect();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }
}
