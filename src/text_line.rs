use std::{error::Error, fmt::Display, ops::Range};

use anyhow::Result;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::terminal::{self, TerminalPos};

#[derive(Clone, Copy, PartialEq, Eq)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    fn width(self) -> u64 {
        match self {
            GraphemeWidth::Half => 1,
            GraphemeWidth::Full => 2,
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

pub struct TextLine {
    fragments: Vec<TextFragment>,
}

pub struct InsertCharResult {
    /// There could be scenarios where an insertion of
    /// a new character results in grapheme clusters
    /// merging together. In those situations, the line
    /// length would not increase, and this value would
    /// be false. Therefore the caret position should
    /// not change.
    ///
    /// Otherwise, if there's a length increase, then
    /// the caret position should change, and this
    /// value would be true.
    pub line_len_increased: bool,
}

#[derive(Debug)]
pub enum InsertCharError {
    InvalidPosition,
}

impl Display for InsertCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for InsertCharError {}

fn get_grapheme_render_replacement<T: AsRef<str>>(grapheme: T) -> Option<(char, GraphemeWidth)> {
    let grapheme = grapheme.as_ref();

    match grapheme {
        " " => None,
        "\t" => Some((' ', GraphemeWidth::Half)),
        _ => {
            if grapheme.trim().is_empty() {
                Some(('␣', GraphemeWidth::Half))
            } else if grapheme.chars().next().is_some_and(char::is_control)
                && grapheme.chars().nth(1).is_none()
            {
                Some(('▯', GraphemeWidth::Half))
            } else if grapheme.width() == 0 {
                Some(('·', GraphemeWidth::Half))
            } else {
                None
            }
        }
    }
}

fn build_fragments_from_string<T: AsRef<str>>(content: T) -> Vec<TextFragment> {
    content
        .as_ref()
        .graphemes(true)
        .map(|grapheme| {
            if let Some((replacement, rendered_width)) = get_grapheme_render_replacement(grapheme) {
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement: Some(replacement),
                }
            } else {
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width: if grapheme.width() <= 1 {
                        GraphemeWidth::Half
                    } else {
                        GraphemeWidth::Full
                    },
                    replacement: None,
                }
            }
        })
        .collect()
}

impl TextLine {
    pub fn new<T: AsRef<str>>(content: T) -> Self {
        Self {
            fragments: build_fragments_from_string(content),
        }
    }

    pub fn get_line_len(&self) -> usize {
        self.fragments.len()
    }

    pub fn get_line_text_width(&self, end_x: usize) -> u64 {
        self.fragments
            .iter()
            .take(end_x)
            .map(|x| x.rendered_width.width())
            .sum()
    }

    pub fn render_line(&self, screen_pos: TerminalPos, text_offset_x: Range<u64>) -> Result<()> {
        let mut current_x = 0;
        let mut fragment_iter = self.fragments.iter();

        let mut chars_to_render = vec![];

        while current_x < text_offset_x.end {
            if let Some(current_fragment) = fragment_iter.next() {
                let next_x = current_x.saturating_add(current_fragment.rendered_width.width());

                if current_x < text_offset_x.start {
                    if next_x > text_offset_x.start {
                        chars_to_render.push("⋯".to_string());
                    }
                } else if next_x > text_offset_x.end {
                    chars_to_render.push("⋯".to_string());
                } else {
                    chars_to_render.push(
                        current_fragment
                            .replacement
                            .map_or(current_fragment.grapheme.to_string(), |replacement| {
                                replacement.to_string()
                            }),
                    );
                }

                current_x = next_x;
            } else {
                // ran out of characters
                break;
            }
        }

        if !chars_to_render.is_empty() {
            terminal::draw_text(
                TerminalPos {
                    x: screen_pos.x,
                    y: screen_pos.y,
                },
                chars_to_render.into_iter().collect::<String>(),
            )?;
        }

        Ok(())
    }

    pub fn insert_character(
        &mut self,
        fragment_idx: usize,
        character: char,
    ) -> Result<InsertCharResult, InsertCharError> {
        if fragment_idx > self.fragments.len() {
            Err(InsertCharError::InvalidPosition)
        } else {
            let old_fragments_len = self.fragments.len();

            let mut new_string = self
                .fragments
                .iter()
                .take(fragment_idx)
                .map(|fragment| fragment.grapheme.clone())
                .collect::<String>();
            new_string.push(character);
            new_string.extend(
                self.fragments
                    .iter()
                    .skip(fragment_idx)
                    .map(|fragment| fragment.grapheme.clone()),
            );

            self.fragments = build_fragments_from_string(new_string);

            Ok(InsertCharResult {
                line_len_increased: self.fragments.len() > old_fragments_len,
            })
        }
    }

    pub fn remove_character(&mut self, fragment_idx: usize) {
        if fragment_idx < self.fragments.len() {
            let new_string = self
                .fragments
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != fragment_idx)
                .map(|(_, fragment)| fragment.grapheme.clone())
                .collect::<String>();
            self.fragments = build_fragments_from_string(new_string);
        }
    }
}

impl ToString for TextLine {
    fn to_string(&self) -> String {
        self.fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect()
    }
}
