use std::{cmp::Ordering, fmt::Display, ops::Range};

use knap_base::math::{Lossy, Vec2f};
use knap_window::drawer::Drawer;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::{TextColor, TextHighlightLine};

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

// TODO: This now only contains rendering logic, refactor it as such
pub(crate) struct TextLine {
    fragments: Vec<TextFragment>,
    string: String,
}

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
    pub(crate) fn new<T: AsRef<str>>(content: T) -> Self {
        Self {
            fragments: build_fragments_from_string(&content),
            string: content.as_ref().to_string(),
        }
    }

    pub(crate) fn get_line_len(&self) -> usize {
        self.fragments.len()
    }

    pub(crate) fn get_line_text_width(&self, end_x: usize) -> u64 {
        self.fragments
            .iter()
            .take(end_x)
            .map(|x| x.rendered_width.width())
            .sum()
    }

    // TODO: Consider refactoring this in the future, so that we
    // do not have to disable too_many_lines lint
    #[allow(clippy::too_many_lines)]
    pub(crate) fn render_line(
        &self,
        drawer: &mut Drawer,
        screen_pos: Vec2f,
        text_offset_x: Range<u64>,
        highlights: &TextHighlightLine,
    ) {
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
                |mut acc: Vec<(String, u64, Option<TextColor>)>, current| {
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
            grouped_strings.into_iter().fold(
                0u64,
                |x_offset, (string, string_width, highlight_type)| {
                    let next_x_offset = x_offset.saturating_add(string_width);
                    let (foreground, background) = match highlight_type {
                        None => (None, None),
                        Some(TextColor {
                            foreground,
                            background,
                        }) => (foreground, background),
                    };

                    drawer.draw_colored_text(
                        Vec2f {
                            x: screen_pos.x + x_offset.lossy(),
                            y: screen_pos.y,
                        },
                        string,
                        foreground,
                        background,
                    );

                    next_x_offset
                },
            );
        }
    }

    // TODO: Maybe create a FragmentIdx type?
    pub(crate) fn get_fragment_idx_from_byte_idx(&self, byte_idx: usize) -> Option<usize> {
        match byte_idx.cmp(&self.string.len()) {
            Ordering::Less => self
                .fragments
                .iter()
                .position(|fragment| fragment.start_byte_index >= byte_idx),
            Ordering::Equal => Some(self.fragments.len()),
            Ordering::Greater => None,
        }
    }

    pub(crate) fn get_byte_idx_from_fragment_idx(&self, fragment_idx: usize) -> Option<usize> {
        match fragment_idx.cmp(&self.fragments.len()) {
            Ordering::Less => Some(self.fragments[fragment_idx].start_byte_index),
            Ordering::Equal => Some(self.string.len()),
            Ordering::Greater => None,
        }
    }
}

impl Display for TextLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
