use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::Range,
};

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
    Keyword,
    BasicType,
    EnumLiteral,
    /// sort of... the tutorial doesn't specify how to verify that
    /// it is actually a legal Rust character
    Character,
    LifetimeSpecifier,
    Comment,
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

    static KEYWORD_TYPES: RefCell<HashSet<String>> = RefCell::new([
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
        "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"
    ].iter().map(|s| (*s).to_string()).collect());

    static BASIC_TYPES: RefCell<HashSet<String>> = RefCell::new([
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64",
        "bool", "char",
        "Option", "Result",
        "String", "str",
        "Vec", "HashMap", "HashSet"
    ].iter().map(|s| (*s).to_string()).collect());

    static ENUM_LITERALS: RefCell<HashSet<String>> = RefCell::new([
        "Some", "None", "Ok", "Err", "true", "false",
    ].iter().map(|s| (*s).to_string()).collect());
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
        // highlight single line comments
        if let Some(single_line_comment_start) = line.as_ref().find("//") {
            highlights.push(Highlight {
                highlight_type: HighlightType::Comment,
                range: single_line_comment_start..(line.as_ref().len()),
            });
        }

        {
            let mut last_seen_quote = None;
            let mut escaped = false;

            // highlight characters
            line.as_ref()
                .split_word_bound_indices()
                .for_each(|(current_idx, current)| match last_seen_quote {
                    None => {
                        if current == "'" {
                            last_seen_quote = Some(current_idx);
                        }
                    }
                    Some(last_seen_idx) => {
                        if current == "\\" {
                            escaped = true;
                        } else {
                            if current == "'" && !escaped {
                                highlights.push(Highlight {
                                    highlight_type: HighlightType::Character,
                                    range: last_seen_idx
                                        ..(current_idx.saturating_add(current.len())),
                                });
                                last_seen_quote = None;
                            } else if current == " " {
                                last_seen_quote = None;
                            }
                            escaped = false;
                        }
                    }
                });
        }

        {
            // highlight lifetime specifier
            let mut iter = line.as_ref().split_word_bound_indices().peekable();

            while let Some((current_idx, current)) = iter.next() {
                if current == "'" {
                    if let Some((next_idx, next)) = iter.peek() {
                        if next
                            .chars()
                            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                        {
                            highlights.push(Highlight {
                                highlight_type: HighlightType::LifetimeSpecifier,
                                range: current_idx..(next_idx.saturating_add(next.len())),
                            });
                        }
                    }
                }
            }
        }

        line.as_ref()
            .split_word_bound_indices()
            .for_each(|(byte_idx, word)| {
                let range = byte_idx..(byte_idx.saturating_add(word.len()));
                let highlight_type = if KEYWORD_TYPES.with_borrow(|set| set.contains(word)) {
                    Some(HighlightType::Keyword)
                } else if BASIC_TYPES.with_borrow(|set| set.contains(word)) {
                    Some(HighlightType::BasicType)
                } else if ENUM_LITERALS.with_borrow(|set| set.contains(word)) {
                    Some(HighlightType::EnumLiteral)
                } else if NUMBER_REGEX.with_borrow(|regex| regex.is_match(word))
                    || BINARY_REGEX.with_borrow(|regex| regex.is_match(word))
                    || OCTAL_REGEX.with_borrow(|regex| regex.is_match(word))
                    || HEXADECIMAL_REGEX.with_borrow(|regex| regex.is_match(word))
                {
                    Some(HighlightType::Number)
                } else {
                    None
                };

                if let Some(highlight_type) = highlight_type {
                    highlights.push(Highlight {
                        highlight_type,
                        range,
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
