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
