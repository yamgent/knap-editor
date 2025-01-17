use std::ops::Range;

use anyhow::Result;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    math::ToUsizeClamp,
    terminal::{self, TerminalPos},
};

pub struct Buffer {
    content: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { content: vec![] }
    }

    pub fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;

        Ok(Self {
            content: content.lines().map(ToString::to_string).collect(),
        })
    }

    pub fn get_line_len(&self, idx: usize) -> usize {
        self.content.get(idx).map_or(0, |line| {
            UnicodeSegmentation::graphemes(line.as_str(), true).count()
        })
    }

    pub fn get_total_lines(&self) -> usize {
        self.content.len()
    }

    pub fn render_line(
        &self,
        line_idx: usize,
        screen_pos: TerminalPos,
        text_offset_x: Range<u64>,
    ) -> Result<()> {
        match self.content.get(line_idx) {
            Some(line) => terminal::draw_text(
                screen_pos,
                UnicodeSegmentation::graphemes(line.as_str(), true)
                    .skip(text_offset_x.start.to_usize_clamp())
                    .take(
                        text_offset_x
                            .end
                            .saturating_sub(text_offset_x.start)
                            .to_usize_clamp(),
                    )
                    .collect::<String>(),
            ),
            None => terminal::draw_text(screen_pos, "~"),
        }
    }
}
