mod line;

use std::{fs::read_to_string, io};

pub(crate) use line::Line;

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

    pub fn line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}
