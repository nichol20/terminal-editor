use std::{
    fs::read_to_string,
    io,
    ops::{Bound, RangeBounds},
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Line {
    pub text: String,
}

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            text: String::from(line_str),
        }
    }

    pub fn get<R>(&self, range: R) -> String
    where
        R: RangeBounds<usize>,
    {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let len = graphemes.len();

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.saturating_add(1),
            Bound::Unbounded => 0,
        }
        .min(len);

        let end = match range.end_bound() {
            Bound::Included(&n) => n.saturating_add(1),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        }
        .min(len);

        graphemes[start..end.max(start)].concat()
    }

    pub fn len(&self) -> usize {
        self.text.graphemes(true).count()
    }
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
