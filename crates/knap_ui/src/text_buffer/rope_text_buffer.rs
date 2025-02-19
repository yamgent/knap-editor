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
            let last_char_is_newline = self.rope.char(
                self.rope
                    .line_to_char(line_idx.saturating_add(1))
                    .saturating_sub(1),
            ) == '\n';
            Some(
                self.rope
                    .line(line_idx)
                    .len_bytes()
                    .saturating_sub(if last_char_is_newline { 1 } else { 0 }),
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
        // TODO: This method can crash if we insert characters beyond the last line in the editor,
        // but it is actually legal to do so.
        match pos.line.cmp(&self.rope.len_lines()) {
            Ordering::Greater => Err(InsertCharError::InvalidLinePosition),
            Ordering::Equal => {
                self.rope.insert_char(self.rope.len_chars(), ch);
                Ok(())
            }
            Ordering::Less => {
                let line_byte = self.rope.line_to_byte(pos.line);
                let char_byte = line_byte.saturating_add(pos.byte);

                if char_byte > self.rope.len_bytes() {
                    return Err(InsertCharError::InvalidLinePosition);
                }

                let char_idx = self.rope.byte_to_char(char_byte);

                if char_idx >= self.rope.line_to_char(pos.line.saturating_add(1)) {
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
        // TODO: Implement
        todo!()
    }
}
