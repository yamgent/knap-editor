use std::{cell::RefCell, collections::HashMap, ops::Range};

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    buffer::{Buffer, FileType},
    math::{ToUsizeClamp, Vec2u},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightType {
    Number,
    SearchMatch,
    SearchCursor,
}

pub struct Highlight {
    highlight_type: HighlightType,
    range: Range<usize>,
}

pub struct Highlights {
    highlights: Vec<Highlight>,
}

impl Highlights {
    pub fn new() -> Self {
        Self { highlights: vec![] }
    }

    pub fn get_highlight_at(&self, byte_idx: usize) -> Option<HighlightType> {
        self.highlights
            .iter()
            .find(|highlight| highlight.range.contains(&byte_idx))
            .map(|highlight| highlight.highlight_type)
    }
}

pub struct HighlightInfo {
    line_info: HashMap<usize, Highlights>,
    file_type: FileType,
}

thread_local! {
    static NUMBER_REGEX: RefCell<Regex> =
        RefCell::new(Regex::new(r"^\d+(_\d+)*(\.\d+)?(e\d+)?$").expect("valid regex expression"));

    static BINARY_REGEX: RefCell<Regex> =
        RefCell::new(Regex::new(r"^0[bB][01]+$").expect("valid regex expression"));

    static OCTAL_REGEX: RefCell<Regex> =
        RefCell::new(Regex::new(r"^0[oO][01234567]+$").expect("valid regex expression"));

    static HEXADECIMAL_REGEX: RefCell<Regex> =
        RefCell::new(Regex::new(r"^0[xX][\dabcdefABCDEF]+$").expect("valid regex expression"));
}

fn get_highlights_for_line<T: AsRef<str>>(
    line: T,
    file_type: FileType,
    search_text: Option<&String>,
    search_cursor_x_pos: Option<u64>,
) -> Highlights {
    let mut highlights = match search_text {
        Some(search_text) => line
            .as_ref()
            .match_indices(search_text)
            .map(|entries| Highlight {
                highlight_type: if search_cursor_x_pos.is_some()
                    && entries.0 == search_cursor_x_pos.unwrap_or_default().to_usize_clamp()
                {
                    HighlightType::SearchCursor
                } else {
                    HighlightType::SearchMatch
                },
                range: entries.0..(entries.0.saturating_add(search_text.len())),
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };

    if matches!(file_type, FileType::Rust) {
        line.as_ref()
            .split_word_bound_indices()
            .for_each(|(byte_idx, word)| {
                if NUMBER_REGEX.with_borrow(|regex| regex.is_match(word))
                    || BINARY_REGEX.with_borrow(|regex| regex.is_match(word))
                    || OCTAL_REGEX.with_borrow(|regex| regex.is_match(word))
                    || HEXADECIMAL_REGEX.with_borrow(|regex| regex.is_match(word))
                {
                    highlights.push(Highlight {
                        highlight_type: HighlightType::Number,
                        range: byte_idx..(byte_idx.saturating_add(word.len())),
                    });
                }
            });
    }

    Highlights { highlights }
}

impl HighlightInfo {
    pub fn new() -> Self {
        Self {
            line_info: HashMap::new(),
            file_type: FileType::PlainText,
        }
    }

    pub fn update_file_type(&mut self, buffer: &Buffer) {
        self.file_type = buffer.file_type();
        self.regenerate_on_buffer_change(buffer);
    }

    pub fn regenerate_on_search_change<T: AsRef<str>>(
        &mut self,
        buffer: &Buffer,
        search_text: T,
        search_cursor_pos: Vec2u,
    ) {
        self.line_info = (0..buffer.get_total_lines())
            .filter_map(|line_idx| buffer.get_raw_line(line_idx))
            .enumerate()
            .map(|(line_idx, line)| {
                let search_cursor_x_pos = if line_idx == search_cursor_pos.y.to_usize_clamp() {
                    Some(search_cursor_pos.x)
                } else {
                    None
                };
                (
                    line_idx,
                    get_highlights_for_line(
                        line,
                        self.file_type,
                        Some(&search_text.as_ref().to_string()),
                        search_cursor_x_pos,
                    ),
                )
            })
            .collect();
    }

    pub fn regenerate_on_buffer_change(&mut self, buffer: &Buffer) {
        self.line_info = (0..buffer.get_total_lines())
            .filter_map(|line_idx| buffer.get_raw_line(line_idx))
            .map(|line| {
                get_highlights_for_line(
                    line,
                    self.file_type,
                    // buffer change should not happen during search for our current
                    // implementation, so safe to pass in None for now
                    None,
                    None,
                )
            })
            .enumerate()
            .collect();
    }

    pub fn clear_search_highlights(&mut self, buffer: &Buffer) {
        self.regenerate_on_buffer_change(buffer);
    }

    pub fn line_highlight(&self, line_idx: usize) -> Option<&Highlights> {
        self.line_info.get(&line_idx)
    }
}
