use std::cmp::Ordering;

use super::{
    InsertCharError, JoinLineResult, RemoveCharError, SearchDirection, TextBuffer, TextBufferPos,
};

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
}

impl TextBuffer for VecTextBuffer {
    fn contents(&self) -> String {
        self.text.join("\n")
    }

    fn set_contents(&mut self, contents: &str) {
        self.text = contents.lines().map(ToString::to_string).collect();
    }

    fn line(&self, line_idx: usize) -> Option<String> {
        self.text.get(line_idx).map(ToString::to_string)
    }

    fn line_len(&self, line_idx: usize) -> Option<usize> {
        self.text.get(line_idx).map(String::len)
    }

    fn total_lines(&self) -> usize {
        self.text.len()
    }

    fn insert_character_at_pos(
        &mut self,
        pos: TextBufferPos,
        ch: char,
    ) -> Result<(), InsertCharError> {
        if ch == '\n' {
            self.insert_newline_at_pos(pos)
        } else {
            if pos.line == self.text.len() {
                if pos.byte == 0 {
                    self.text.push(ch.to_string());
                    Ok(())
                } else {
                    Err(InsertCharError::InvalidBytePosition)
                }
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

    fn remove_character_at_pos(&mut self, pos: TextBufferPos) -> Result<(), RemoveCharError> {
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

    fn find(
        &self,
        search: &str,
        start_pos: TextBufferPos,
        search_direction: SearchDirection,
    ) -> Option<TextBufferPos> {
        if let Some(first_line) = self.text.get(start_pos.line) {
            if start_pos.byte < first_line.len() {
                let first_line_result = match search_direction {
                    SearchDirection::Forward => {
                        first_line[start_pos.byte..]
                            .find(search)
                            .map(|byte| TextBufferPos {
                                line: start_pos.line,
                                byte: start_pos.byte.saturating_add(byte),
                            })
                    }
                    SearchDirection::Backward => {
                        let end_byte = start_pos
                            .byte
                            .saturating_add(search.len().saturating_sub(1))
                            .clamp(0, first_line.len());
                        first_line[..end_byte]
                            .rfind(search)
                            .map(|byte| TextBufferPos {
                                line: start_pos.line,
                                byte,
                            })
                    }
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
                    line.find(search).map(|byte| TextBufferPos {
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
                    line.rfind(search).map(|byte| TextBufferPos {
                        line: line_idx,
                        byte,
                    })
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_contents() {
        // normal
        {
            let mut buffer = VecTextBuffer::new();
            buffer.set_contents("Hello\nWorld!\n\nThe End");
            assert_eq!(
                buffer.text,
                vec![
                    "Hello".to_string(),
                    "World!".to_string(),
                    "".to_string(),
                    "The End".to_string()
                ]
            );
            assert_eq!(buffer.contents(), "Hello\nWorld!\n\nThe End");
        }

        // empty
        {
            let mut buffer = VecTextBuffer::new();
            buffer.set_contents("");
            assert_eq!(buffer.contents(), "");
        }
    }

    #[test]
    fn test_standard_text_buffer_tests() {
        use crate::text_buffer::buffer_tests::do_standard_text_buffer_tests;

        do_standard_text_buffer_tests(&|| VecTextBuffer::new());
    }
}
