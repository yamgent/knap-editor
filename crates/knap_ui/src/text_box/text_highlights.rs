use std::{collections::HashMap, ops::Range};

use knap_base::color::Color;

pub struct TextHighlights {
    pub lines: HashMap<usize, TextHighlightLine>,
}

impl TextHighlights {
    pub fn new() -> Self {
        Self {
            lines: HashMap::new(),
        }
    }

    pub(crate) fn line_highlight(&self, line_idx: usize) -> Option<&TextHighlightLine> {
        self.lines.get(&line_idx)
    }
}

pub struct TextHighlightLine {
    pub blocks: Vec<TextHighlightBlock>,
}

impl TextHighlightLine {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    pub(crate) fn get_highlight_at(&self, byte_idx: usize) -> Option<TextColor> {
        self.blocks
            .iter()
            .find(|highlight| highlight.range.contains(&byte_idx))
            .map(|highlight| highlight.color)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextHighlightBlock {
    pub color: TextColor,
    pub range: Range<usize>,
}

// TODO: This can be part of theme in the future
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextColor {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}
