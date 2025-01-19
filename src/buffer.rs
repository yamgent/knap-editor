use std::ops::Range;

use anyhow::Result;

use crate::{
    terminal::{self, TerminalPos},
    text_line::{InsertCharError, InsertCharResult, TextLine},
};

pub struct Buffer {
    content: Vec<TextLine>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { content: vec![] }
    }

    pub fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;

        Ok(Self {
            content: content.lines().map(TextLine::new).collect(),
        })
    }

    pub fn get_line_len(&self, line_idx: usize) -> usize {
        self.content.get(line_idx).map_or(0, TextLine::get_line_len)
    }

    pub fn get_line_text_width(&self, line_idx: usize, end_x: usize) -> u64 {
        self.content
            .get(line_idx)
            .map_or(0, |line| line.get_line_text_width(end_x))
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
            Some(line) => line.render_line(screen_pos, text_offset_x),
            None => terminal::draw_text(screen_pos, "~"),
        }
    }

    pub fn insert_character(
        &mut self,
        line_idx: usize,
        fragment_idx: usize,
        character: char,
    ) -> Result<InsertCharResult, InsertCharError> {
        if line_idx == self.content.len() {
            self.content.push(TextLine::new(character.to_string()));
            Ok(InsertCharResult {
                line_len_increased: true,
            })
        } else {
            match self.content.get_mut(line_idx) {
                Some(line) => line.insert_character(fragment_idx, character),
                None => Err(InsertCharError::InvalidPosition),
            }
        }
    }
}
