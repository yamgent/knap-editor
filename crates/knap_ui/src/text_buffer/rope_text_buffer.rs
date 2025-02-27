use std::cmp::Ordering;

use ropey::Rope;

use super::{InsertCharError, RemoveCharError, SearchDirection, TextBuffer, TextBufferPos};

/// A text buffer that is stored in a rope.
///
/// This is currently backed by the `ropey` implementation.
///
/// - Create: O(n)
/// - Retrieve: O(lg n)
/// - Insert: O(lg n)
/// - Remove: O(m + lg n) [m is the length of the range to remove]
pub struct RopeTextBuffer {
    rope: Rope,
}

impl RopeTextBuffer {
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    fn char_idx(&self, buffer_pos: TextBufferPos) -> usize {
        let line_byte = self.rope.line_to_byte(buffer_pos.line);
        let char_byte = line_byte.saturating_add(buffer_pos.byte);
        self.rope.byte_to_char(char_byte)
    }
}

impl TextBuffer for RopeTextBuffer {
    fn contents(&self) -> String {
        self.rope.to_string()
    }

    fn set_contents(&mut self, contents: &str) {
        // TODO: How to handle a mixture of "\r\n" and "\n"?
        self.rope = Rope::from_str(&contents.replace("\r\n", "\n"));
    }

    fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            let mut final_line = self.rope.line(line_idx).to_string();

            if final_line.ends_with('\n') {
                final_line.pop();
            }

            Some(final_line)
        } else {
            None
        }
    }

    fn line_len(&self, line_idx: usize) -> Option<usize> {
        if line_idx < self.rope.len_lines() {
            let last_char_pos = self
                .rope
                .line_to_char(line_idx.saturating_add(1))
                .saturating_sub(1);

            let last_char_is_newline =
                last_char_pos < self.rope.len_chars() && self.rope.char(last_char_pos) == '\n';
            Some(
                self.rope
                    .line(line_idx)
                    .len_bytes()
                    .saturating_sub(if last_char_is_newline { "\n".len() } else { 0 }),
            )
        } else {
            None
        }
    }

    fn total_lines(&self) -> usize {
        self.rope.len_lines()
    }

    fn insert_character_at_pos(
        &mut self,
        pos: TextBufferPos,
        ch: char,
    ) -> Result<(), InsertCharError> {
        match pos.line.cmp(&self.rope.len_lines()) {
            Ordering::Greater => Err(InsertCharError::InvalidLinePosition),
            Ordering::Equal => {
                if pos.byte == 0 {
                    self.rope.insert_char(self.rope.len_chars(), '\n');
                    self.rope.insert_char(self.rope.len_chars(), ch);
                    Ok(())
                } else {
                    Err(InsertCharError::InvalidBytePosition)
                }
            }
            Ordering::Less => {
                let line_byte = self.rope.line_to_byte(pos.line);
                let char_byte = line_byte.saturating_add(pos.byte);

                if char_byte > self.rope.len_bytes() {
                    return Err(InsertCharError::InvalidLinePosition);
                }

                let char_idx = self.rope.byte_to_char(char_byte);

                if char_idx >= self.rope.line_to_char(pos.line.saturating_add(1))
                    && pos.line.saturating_add(1) != self.rope.len_lines()
                {
                    return Err(InsertCharError::InvalidBytePosition);
                }

                self.rope.insert_char(char_idx, ch);
                Ok(())
            }
        }
    }

    fn remove_character_at_pos(&mut self, pos: TextBufferPos) -> Result<(), RemoveCharError> {
        if pos.line >= self.rope.len_lines() {
            return Err(RemoveCharError::InvalidLinePosition);
        }

        let line_byte = self.rope.line_to_byte(pos.line);
        let char_byte = line_byte.saturating_add(pos.byte);

        if char_byte >= self.rope.len_bytes() {
            return Err(RemoveCharError::InvalidBytePosition);
        }

        let char_idx = self.rope.byte_to_char(char_byte);

        if char_idx >= self.rope.line_to_char(pos.line.saturating_add(1)) {
            return Err(RemoveCharError::InvalidBytePosition);
        }

        self.rope.remove(char_idx..=char_idx);
        Ok(())
    }

    fn find(
        &self,
        search: &str,
        start_pos: TextBufferPos,
        search_direction: SearchDirection,
    ) -> Option<TextBufferPos> {
        if start_pos.line >= self.total_lines() {
            return None;
        }

        if start_pos.byte >= self.line_len(start_pos.line).unwrap_or(0) {
            return None;
        }

        let search_chars_len = search.chars().count();
        let start_char_idx = self.char_idx(start_pos);

        let substring_matches_search = |char_idx: &usize| {
            self.rope
                .slice(char_idx..&char_idx.saturating_add(search_chars_len))
                == search
        };

        match search_direction {
            SearchDirection::Forward => (start_char_idx
                ..self.rope.len_chars().saturating_sub(search_chars_len))
                .find(substring_matches_search)
                .or_else(|| (0..start_char_idx).find(substring_matches_search)),
            SearchDirection::Backward => (0..start_char_idx)
                .rev()
                .find(substring_matches_search)
                .or_else(|| {
                    (start_char_idx..self.rope.len_chars().saturating_sub(search_chars_len))
                        .rev()
                        .find(substring_matches_search)
                }),
        }
        .map(|result_char_idx| {
            let line = self.rope.char_to_line(result_char_idx);
            let byte = self
                .rope
                .char_to_byte(result_char_idx)
                .saturating_sub(self.rope.line_to_byte(line));
            TextBufferPos { line, byte }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_contents() {
        // normal
        {
            let mut buffer = RopeTextBuffer::new();
            buffer.set_contents("Hello\nWorld!\n\nThe End");
            assert_eq!(buffer.rope, Rope::from_str("Hello\nWorld!\n\nThe End"));
            assert_eq!(buffer.contents(), "Hello\nWorld!\n\nThe End");
        }

        // empty
        {
            let mut buffer = RopeTextBuffer::new();
            buffer.set_contents("");
            assert_eq!(buffer.rope, Rope::from_str(""));
            assert_eq!(buffer.contents(), "");
        }
    }

    #[test]
    fn test_standard_text_buffer_tests() {
        use crate::text_buffer::buffer_tests::do_standard_text_buffer_tests;

        do_standard_text_buffer_tests(&|| RopeTextBuffer::new());
    }

    #[test]
    fn test_inserting_characters_at_last_line() {
        {
            let mut buffer = RopeTextBuffer {
                rope: Rope::from_str("Line 1\nLine 2\nLine 3"),
            };

            {
                let result =
                    buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 0 }, 'X');
                assert_eq!(result, Ok(()));
                assert_eq!(buffer.contents(), "Line 1\nLine 2\nLine 3\nX");
            }

            {
                let result =
                    buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 1 }, 'Y');
                assert_eq!(result, Ok(()));
                assert_eq!(buffer.contents(), "Line 1\nLine 2\nLine 3\nXY");
            }
        }

        {
            let mut buffer = RopeTextBuffer {
                rope: Rope::from_str("Line 1\nLine 2\nLine 3\n"),
            };

            let result = buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 0 }, 'X');
            assert_eq!(result, Ok(()));
            assert_eq!(buffer.contents(), "Line 1\nLine 2\nLine 3\nX");
        }
    }
}
