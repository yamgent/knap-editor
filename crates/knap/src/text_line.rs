use std::{error::Error, fmt::Display, ops::Range};

use anyhow::Result;
use crossterm::style::Color;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    highlighter::{HighlightType, Highlights},
    math::ToU16Clamp,
    search::SearchDirection,
    terminal::{self, TerminalPos},
};

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
    start_byte_index: usize,
}

pub struct TextLine {
    fragments: Vec<TextFragment>,
    string: String,
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
        .grapheme_indices(true)
        .map(|(start_byte_index, grapheme)| {
            if let Some((replacement, rendered_width)) = get_grapheme_render_replacement(grapheme) {
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement: Some(replacement),
                    start_byte_index,
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
                    start_byte_index,
                }
            }
        })
        .collect()
}

impl TextLine {
    pub fn new<T: AsRef<str>>(content: T) -> Self {
        Self {
            fragments: build_fragments_from_string(&content),
            string: content.as_ref().to_string(),
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

    // TODO: Consider refactoring this in the future, so that we
    // do not have to disable too_many_lines lint
    #[allow(clippy::too_many_lines)]
    pub fn render_line(
        &self,
        screen_pos: TerminalPos,
        text_offset_x: Range<u64>,
        highlights: &Highlights,
    ) -> Result<()> {
        let mut current_x = 0;
        let mut fragment_iter = self.fragments.iter();

        let mut chars_to_render = vec![];

        while current_x < text_offset_x.end {
            if let Some(current_fragment) = fragment_iter.next() {
                let next_x = current_x.saturating_add(current_fragment.rendered_width.width());

                if current_x < text_offset_x.start {
                    if next_x > text_offset_x.start {
                        chars_to_render.push(("⋯".to_string(), 1, None));
                    }
                } else if next_x > text_offset_x.end {
                    chars_to_render.push(("⋯".to_string(), 1, None));
                } else {
                    chars_to_render.push((
                        current_fragment
                            .replacement
                            .map_or(current_fragment.grapheme.to_string(), |replacement| {
                                replacement.to_string()
                            }),
                        current_fragment.rendered_width.width(),
                        highlights.get_highlight_at(current_fragment.start_byte_index),
                    ));
                }

                current_x = next_x;
            } else {
                // ran out of characters
                break;
            }
        }

        if !chars_to_render.is_empty() {
            let grouped_strings = chars_to_render.into_iter().fold(
                vec![],
                |mut acc: Vec<(String, u64, Option<HighlightType>)>, current| {
                    let mut insert_new = true;

                    if let Some(last_entry) = acc.last_mut() {
                        if last_entry.2 == current.2 {
                            last_entry.0.push_str(&current.0);
                            last_entry.1 = last_entry.1.saturating_add(current.1);
                            insert_new = false;
                        }
                    }

                    if insert_new {
                        acc.push(current);
                    }

                    acc
                },
            );
            grouped_strings
                .into_iter()
                .fold(
                    (0u64, Ok(())),
                    |(x_offset, recent_result), (string, string_width, highlight_type)| {
                        if recent_result.is_err() {
                            (0, recent_result)
                        } else {
                            let next_x_offset = x_offset.saturating_add(string_width);
                            let (foreground, background) = match highlight_type {
                                None => (None, None),
                                Some(HighlightType::SearchMatch) => {
                                    (Some(Color::Black), Some(Color::Yellow))
                                }
                                Some(HighlightType::SearchCursor) => {
                                    (Some(Color::Black), Some(Color::Blue))
                                }
                                Some(HighlightType::Number) => (Some(Color::DarkRed), None),
                                Some(HighlightType::Keyword) => (Some(Color::Blue), None),
                                Some(HighlightType::BasicType) => (Some(Color::Green), None),
                                Some(HighlightType::EnumLiteral) => (Some(Color::DarkBlue), None),
                                Some(HighlightType::Character) => (Some(Color::DarkYellow), None),
                                Some(HighlightType::LifetimeSpecifier) => {
                                    (Some(Color::DarkYellow), None)
                                }
                                Some(HighlightType::Comment) => (Some(Color::DarkGreen), None),
                            };

                            (
                                next_x_offset,
                                terminal::draw_colored_text(
                                    TerminalPos {
                                        x: screen_pos.x.saturating_add(x_offset.to_u16_clamp()),
                                        y: screen_pos.y,
                                    },
                                    string,
                                    foreground,
                                    background,
                                ),
                            )
                        }
                    },
                )
                .1?;
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

            *self = Self::new(new_string);

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
            *self = Self::new(new_string);
        }
    }

    pub fn split_off(&mut self, fragment_idx: usize) -> Self {
        let left = self
            .fragments
            .iter()
            .take(fragment_idx)
            .map(|fragment| fragment.grapheme.clone())
            .collect::<String>();
        let right = self
            .fragments
            .iter()
            .skip(fragment_idx)
            .map(|fragment| fragment.grapheme.clone())
            .collect::<String>();

        *self = Self::new(left);
        Self::new(right)
    }

    fn get_fragment_idx_from_byte_idx(&self, byte_idx: usize) -> Option<usize> {
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_index >= byte_idx)
    }

    fn get_byte_idx_from_fragment_idx(&self, fragment_idx: usize) -> Option<usize> {
        self.fragments
            .get(fragment_idx)
            .map(|fragment| fragment.start_byte_index)
    }

    pub fn find<T: AsRef<str>>(
        &self,
        search: T,
        start_from_fragment_idx: Option<usize>,
        search_direction: SearchDirection,
    ) -> Option<usize> {
        let start_byte_idx = match start_from_fragment_idx {
            Some(start_from_fragment_idx) => {
                self.get_byte_idx_from_fragment_idx(start_from_fragment_idx)?
            }
            None => match search_direction {
                SearchDirection::Forward => 0,
                SearchDirection::Backward => self.string.len(),
            },
        };
        let all_indices = self
            .string
            .match_indices(search.as_ref())
            .map(|entries| entries.0)
            .collect::<Vec<_>>();

        match search_direction {
            SearchDirection::Forward => all_indices
                .iter()
                .find(|byte_idx| **byte_idx >= start_byte_idx)
                .and_then(|byte_idx| self.get_fragment_idx_from_byte_idx(*byte_idx)),
            SearchDirection::Backward => all_indices
                .iter()
                .rev()
                .find(|byte_idx| **byte_idx < start_byte_idx)
                .and_then(|byte_idx| self.get_fragment_idx_from_byte_idx(*byte_idx)),
        }
    }
}

impl Display for TextLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
