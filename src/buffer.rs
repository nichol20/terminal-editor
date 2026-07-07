use std::{fs::read_to_string, io};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        self.lines = read_to_string(path)?.lines().map(String::from).collect();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }
}
