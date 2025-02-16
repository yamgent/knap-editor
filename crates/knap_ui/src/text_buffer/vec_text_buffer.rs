use std::fmt::Display;

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
    pub fn new() -> Self {
        Self { text: vec![] }
    }

    pub fn set_contents<T: AsRef<str>>(&mut self, contents: T) {
        self.text = contents.as_ref().lines().map(ToString::to_string).collect();
    }

    pub fn line(&self, line_idx: usize) -> Option<&String> {
        self.text.get(line_idx)
    }

    pub fn lines_len(&self) -> usize {
        self.text.len()
    }

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

    pub fn insert_newline_at_pos(&mut self, pos: TextBufferPos) -> Result<(), InsertCharError> {
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

    pub fn remove_character_at_pos(&mut self, pos: TextBufferPos) -> Result<(), RemoveCharError> {
        match self.text.get_mut(pos.line) {
            Some(line) => {
                if pos.byte < line.len() {
                    line.remove(pos.byte);
                    Ok(())
                } else {
                    Err(RemoveCharError::InvalidBytePosition)
                }
            }
            None => Err(RemoveCharError::InvalidLinePosition),
        }
    }

    pub fn join_line_with_below_line(&mut self, line: usize) -> JoinLineResult {
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
