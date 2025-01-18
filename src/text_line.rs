use std::ops::Range;

use anyhow::Result;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    math::ToU16Clamp,
    terminal::{self, TerminalPos},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    fn width(&self) -> u64 {
        match self {
            GraphemeWidth::Half => 1,
            GraphemeWidth::Full => 2,
        }
    }

    fn increment(&self, x: u64) -> u64 {
        x.saturating_add(self.width())
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

impl TextLine {
    pub fn new<T: AsRef<str>>(content: T) -> Self {
        Self {
            fragments: UnicodeSegmentation::graphemes(content.as_ref(), true)
                .map(|grapheme| TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width: if grapheme.width() <= 1 {
                        GraphemeWidth::Half
                    } else {
                        GraphemeWidth::Full
                    },
                    replacement: if grapheme == "\x1b" { Some('·') } else { None },
                })
                .collect(),
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
        let mut current_render_x = screen_pos.x;
        let mut fragment_iter = self.fragments.iter();

        while current_x < text_offset_x.end {
            if let Some(current_fragment) = fragment_iter.next() {
                let next_x = current_fragment.rendered_width.increment(current_x);

                if current_x < text_offset_x.start {
                    if next_x > text_offset_x.start {
                        terminal::draw_text(
                            TerminalPos {
                                x: current_render_x,
                                y: screen_pos.y,
                            },
                            "⋯",
                        )?;
                        current_render_x = current_render_x.saturating_add(1);
                    }
                } else {
                    if next_x > text_offset_x.end {
                        terminal::draw_text(
                            TerminalPos {
                                x: current_render_x,
                                y: screen_pos.y,
                            },
                            "⋯",
                        )?;
                        current_render_x = current_render_x.saturating_add(1);
                    } else {
                        terminal::draw_text(
                            TerminalPos {
                                x: current_render_x,
                                y: screen_pos.y,
                            },
                            current_fragment
                                .replacement
                                .map_or(current_fragment.grapheme.to_string(), |replacement| {
                                    replacement.to_string()
                                }),
                        )?;
                        current_render_x = current_render_x
                            .saturating_add(current_fragment.rendered_width.width().to_u16_clamp());
                    }
                }

                current_x = next_x;
            } else {
                // ran out of characters
                break;
            }
        }

        Ok(())
    }
}
