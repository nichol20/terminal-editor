use std::{fs::read_to_string, io};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(&mut self, path: &str) -> io::Result<()> {
        let contents = read_to_string(path)?;
        for line in contents.lines() {
            self.lines.push(String::from(line));
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }
}
