use std::{cmp::Ordering, fmt::Display};

use super::{InsertCharError, JoinLineResult, RemoveCharError, SearchDirection, TextBufferPos};

/// A text buffer that is stored in a vector of strings.
///
/// This is a basic text buffer, and is not very efficient.
/// But if the content is very small, this might be more worth
/// it than other more fancy text buffers that are usually
/// more efficient for larger content.
///
/// - Create: O(n)
/// - Retrieve: O(1)
/// - Insert: O(n)
/// - Remove: O(n)
pub struct VecTextBuffer {
    text: Vec<String>,
}

impl VecTextBuffer {
    /// Create a new text buffer with no contents.
    pub fn new() -> Self {
        Self { text: vec![] }
    }

    /// Completely replace the contents of the text buffer.
    pub fn set_contents<T: AsRef<str>>(&mut self, contents: T) {
        self.text = contents.as_ref().lines().map(ToString::to_string).collect();
    }

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
    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.text.get(line_idx).map(ToString::to_string)
    }

    /// Get the length of a specific line.
    pub fn line_len(&self, line_idx: usize) -> Option<usize> {
        self.text.get(line_idx).map(String::len)
    }

    /// Get the total number of lines in the text buffer.
    pub fn total_lines(&self) -> usize {
        self.text.len()
    }

    /// Insert a character at a specific position.
    ///
    /// Inserting a newline character will either insert a new line,
    /// or break up an existing line into two lines, depending on the
    /// position of `pos`.
    pub fn insert_character_at_pos(
        &mut self,
        pos: TextBufferPos,
        ch: char,
    ) -> Result<(), InsertCharError> {
        if ch == '\n' {
            self.insert_newline_at_pos(pos)
        } else {
            if pos.line == self.text.len() {
                self.text.push(ch.to_string());
                Ok(())
            } else {
                match self.text.get_mut(pos.line) {
                    Some(line) => {
                        if pos.byte <= line.len() {
                            line.insert(pos.byte, ch);
                            Ok(())
                        } else {
                            Err(InsertCharError::InvalidBytePosition)
                        }
                    }
                    None => Err(InsertCharError::InvalidLinePosition),
                }
            }
        }
    }

    fn insert_newline_at_pos(&mut self, pos: TextBufferPos) -> Result<(), InsertCharError> {
        match self.text.get_mut(pos.line) {
            Some(line) => {
                let right = line.split_off(pos.byte);
                self.text.insert(pos.line.saturating_add(1), right);
                Ok(())
            }
            None => {
                if pos.line == self.text.len() {
                    self.text.push(String::new());
                    Ok(())
                } else {
                    Err(InsertCharError::InvalidLinePosition)
                }
            }
        }
    }

    /// Remove a character at a specific position.
    ///
    /// If the position is directly after the last non-newline character of a line,
    /// (in which such a position would HAVE been the newline character if it is "visible"),
    /// the line will be joined with the next line.
    pub fn remove_character_at_pos(&mut self, pos: TextBufferPos) -> Result<(), RemoveCharError> {
        match self.text.get_mut(pos.line) {
            Some(line) => match pos.byte.cmp(&line.len()) {
                Ordering::Less => {
                    line.remove(pos.byte);
                    Ok(())
                }
                Ordering::Equal => {
                    self.join_line_with_below_line(pos.line);
                    Ok(())
                }
                Ordering::Greater => Err(RemoveCharError::InvalidBytePosition),
            },
            None => Err(RemoveCharError::InvalidLinePosition),
        }
    }

    fn join_line_with_below_line(&mut self, line: usize) -> JoinLineResult {
        let mut new_line_string = None;

        if let Some(first_line) = self.text.get(line) {
            if let Some(second_line) = self.text.get(line.saturating_add(1)) {
                let mut final_string = first_line.to_string();
                final_string.push_str(&second_line.to_string());
                new_line_string = Some(final_string);
            }
        }

        if let Some(new_line) = new_line_string {
            *self
                .text
                .get_mut(line)
                .expect("line to exist as new_line_string contains line") = new_line;
            self.text.remove(line.saturating_add(1));
            JoinLineResult::Joined
        } else {
            JoinLineResult::NotJoined
        }
    }

    /// Find a substring in the text buffer.
    ///
    /// This function will search for the first occurrence of `search`
    /// in the text buffer, starting from the position specified by `start_pos`.
    ///
    /// `search_direction` specifies the direction of the search.
    ///
    /// Note that if `search` contains a newline the behavior is undefined
    /// (text buffer implementations are not required to honor the newline).
    pub fn find<T: AsRef<str>>(
        &self,
        search: T,
        start_pos: TextBufferPos,
        search_direction: SearchDirection,
    ) -> Option<TextBufferPos> {
        if let Some(first_line) = self.text.get(start_pos.line) {
            if start_pos.byte < first_line.len() {
                let first_line_result = match search_direction {
                    SearchDirection::Forward => first_line[start_pos.byte..]
                        .find(search.as_ref())
                        .map(|byte| TextBufferPos {
                            line: start_pos.line,
                            byte: start_pos.byte.saturating_add(byte),
                        }),
                    SearchDirection::Backward => first_line[..start_pos.byte]
                        .rfind(search.as_ref())
                        .map(|byte| TextBufferPos {
                            line: start_pos.line,
                            byte,
                        }),
                };

                if first_line_result.is_some() {
                    return first_line_result;
                }
            }
        }

        match search_direction {
            SearchDirection::Forward => self
                .text
                .iter()
                .enumerate()
                .cycle()
                .skip(start_pos.line.saturating_add(1))
                .take(self.text.len().saturating_sub(1))
                .find_map(|(line_idx, line)| {
                    line.find(search.as_ref()).map(|byte| TextBufferPos {
                        line: line_idx,
                        byte,
                    })
                }),
            SearchDirection::Backward => self
                .text
                .iter()
                .enumerate()
                .rev()
                .cycle()
                .skip(self.text.len().saturating_sub(start_pos.line))
                .take(self.text.len().saturating_sub(1))
                .find_map(|(line_idx, line)| {
                    line.rfind(search.as_ref()).map(|byte| TextBufferPos {
                        line: line_idx,
                        byte,
                    })
                }),
        }
    }
}

impl Display for VecTextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = self.text.join("\n");
        write!(f, "{}", content)
    }
}
