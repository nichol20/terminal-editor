use std::ops::{Bound, RangeBounds};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub enum GraphemeWidth {
    Half,
    Full,
}

pub struct TextFragment {
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
}

impl TextFragment {
    pub fn from(grapheme_str: &str) -> Self {
        let grapheme_width = grapheme_str.width();
        let replacement = match grapheme_str {
            "\t" => Some(' '),
            // non-zero width whitespace
            //_ if grapheme_width > 0 && grapheme_str.trim().is_empty() => Some('␣'),
            _ => {
                let mut chars = grapheme_str.chars();
                match (chars.next(), chars.next()) {
                    // Exactly one control character
                    (Some(ch), None) if ch.is_control() => Some('▯'),

                    // Exactly one non-control character (zero-width)
                    (Some(_), None) if grapheme_width == 0 => Some('·'),

                    // Multiple characters or normal text
                    _ => None,
                }
            }
        };

        Self {
            grapheme: grapheme_str.to_string(),
            rendered_width: match grapheme_width {
                0 | 1 => GraphemeWidth::Half,
                _ => GraphemeWidth::Full,
            },
            replacement,
        }
    }

    pub fn get_grapheme_width(&self) -> usize {
        match self.rendered_width {
            GraphemeWidth::Half => 1,
            GraphemeWidth::Full => 2,
        }
    }
}

pub struct Line {
    pub content: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let graphemes = line_str.graphemes(true);

        Self {
            content: graphemes.map(TextFragment::from).collect(),
        }
    }

    pub fn get<R>(&self, range: R) -> &[TextFragment]
    where
        R: RangeBounds<usize>,
    {
        let len = self.content.len();

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

        &self.content[start..end.max(start)]
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn total_width(&self) -> usize {
        self.content
            .iter()
            .map(TextFragment::get_grapheme_width)
            .sum()
    }

    pub fn total_width_until(&self, x: usize) -> usize {
        self.get(..x)
            .iter()
            .map(TextFragment::get_grapheme_width)
            .sum()
    }
}
