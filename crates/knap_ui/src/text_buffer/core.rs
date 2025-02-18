use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TextBufferPos {
    pub line: usize,
    /// the byte index of the character in the line
    pub byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InsertCharError {
    InvalidLinePosition,
    InvalidBytePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoveCharError {
    InvalidLinePosition,
    InvalidBytePosition,
}

impl Display for InsertCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for RemoveCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for InsertCharError {}

impl Error for RemoveCharError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinLineResult {
    Joined,
    NotJoined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// A type that can be used to store and manipulate text.
pub trait TextBuffer {
    /// Get the entire contents of the text buffer.
    fn contents(&self) -> String;

    /// Completely replace the contents of the text buffer.
    fn set_contents(&mut self, contents: &str);

    /// Get the contents of a specific line.
    ///
    /// Note that this is expensive due to the need to return
    /// the value as a String. As much as possible, use alternate
    /// methods wherever available (e.g. to get the length of a
    /// line, it would be better to call `line_len()` directly,
    /// instead of `line().map(String::len)`).
    ///
    /// Why do we need to return a String here, instead of
    /// &str or &String?
    ///
    /// We cannot return &str or &String, because not all text
    /// buffers can guarantee that the contents of a line is in
    /// a contiguous block of memory.
    fn line(&self, line_idx: usize) -> Option<String>;

    /// Get the length of a specific line.
    fn line_len(&self, line_idx: usize) -> Option<usize>;

    /// Get the total number of lines in the text buffer.
    fn total_lines(&self) -> usize;

    /// Insert a character at a specific position.
    ///
    /// Inserting a newline character will either insert a new line,
    /// or break up an existing line into two lines, depending on the
    /// position of `pos`.
    fn insert_character_at_pos(
        &mut self,
        pos: TextBufferPos,
        ch: char,
    ) -> Result<(), InsertCharError>;

    /// Remove a character at a specific position.
    ///
    /// If the position is directly after the last non-newline character of a line,
    /// (in which such a position would HAVE been the newline character if it is "visible"),
    /// the line will be joined with the next line.
    fn remove_character_at_pos(&mut self, pos: TextBufferPos) -> Result<(), RemoveCharError>;

    /// Find a substring in the text buffer.
    ///
    /// This function will search for the first occurrence of `search`
    /// in the text buffer, starting from the position specified by `start_pos`.
    ///
    /// `search_direction` specifies the direction of the search.
    ///
    /// Note that if `search` contains a newline the behavior is undefined
    /// (text buffer implementations are not required to honor the newline).
    fn find(
        &self,
        search: &str,
        start_pos: TextBufferPos,
        search_direction: SearchDirection,
    ) -> Option<TextBufferPos>;
}
