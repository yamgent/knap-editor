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
                            .saturating_add(search.len())
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
        let mut buffer = VecTextBuffer::new();
        buffer.set_contents("Hello\nWorld!");
        assert_eq!(buffer.contents(), "Hello\nWorld!");
    }

    #[test]
    fn test_line() {
        let buffer = VecTextBuffer {
            text: vec!["Hello".to_string(), "World!".to_string()],
        };

        assert_eq!(buffer.line(0), Some("Hello".to_string()));
        assert_eq!(buffer.line(1), Some("World!".to_string()));
        assert_eq!(buffer.line(2), None);
    }

    #[test]
    fn test_line_len() {
        let buffer = VecTextBuffer {
            text: vec!["Hello".to_string(), "World!".to_string()],
        };

        assert_eq!(buffer.line_len(0), Some(5));
        assert_eq!(buffer.line_len(1), Some(6));
        assert_eq!(buffer.line_len(2), None);
    }

    #[test]
    fn test_total_lines() {
        let buffer = VecTextBuffer {
            text: vec!["Hello".to_string(), "World!".to_string()],
        };

        assert_eq!(buffer.total_lines(), 2);
    }

    #[test]
    fn test_insert_character_at_pos() {
        let mut buffer = VecTextBuffer::new();

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 0 }, 'a');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "a");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 1 }, 'b');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ab");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 2 }, 'c');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc");
        assert_eq!(buffer.total_lines(), 1);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 3 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\n");
        assert_eq!(buffer.total_lines(), 2);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 0 }, 'd');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\nd");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 1 }, 'e');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\nde");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 1 }, 'f');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\ndfe");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 2 }, 'g');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\ndfge");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 0 }, 'h');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc\nhdfge");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 3 }, 'i');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abci\nhdfge");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 10 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "abci\nhdfge");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 10, byte: 0 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidLinePosition));
        assert_eq!(buffer.contents(), "abci\nhdfge");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 2, byte: 1 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "abci\nhdfge");
        assert_eq!(buffer.total_lines(), 2);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 2, byte: 0 }, 'j');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abci\nhdfge\nj");
        assert_eq!(buffer.total_lines(), 3);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 2 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ab\nci\nhdfge\nj");
        assert_eq!(buffer.total_lines(), 4);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 1 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ab\nci\nhdfge\nj\n");
        assert_eq!(buffer.total_lines(), 5);

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 4, byte: 0 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ab\nci\nhdfge\nj\n\n");
        assert_eq!(buffer.total_lines(), 6);
    }

    #[test]
    fn test_remove_character_at_pos() {
        let mut buffer = VecTextBuffer::new();
        buffer.set_contents("Hello\nWorld!\nAnother\nLine");

        // delete first character of line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 0 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ello\nWorld!\nAnother\nLine");

        // delete newline character, resulting in a join
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 4 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "elloWorld!\nAnother\nLine");

        // delete last character of line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 9 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "elloWorld\nAnother\nLine");

        // fail to delete beyond last line of character (including newline)
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 10 });
        assert_eq!(result, Err(RemoveCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "elloWorld\nAnother\nLine");

        // fail todelete non-existent line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 3, byte: 0 });
        assert_eq!(result, Err(RemoveCharError::InvalidLinePosition));
        assert_eq!(buffer.contents(), "elloWorld\nAnother\nLine");

        // delete character in the middle of the line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 2, byte: 2 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "elloWorld\nAnother\nLie");
    }

    #[test]
    fn test_find() {
        let mut buffer = VecTextBuffer::new();
        buffer.set_contents("this is a text and this is his cat.\nthe next line contains the history.\nand this is the last line.");

        // normal search on the same line
        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 0 },
            SearchDirection::Forward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 0 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 1 },
            SearchDirection::Forward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 19 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 19 },
            SearchDirection::Forward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 19 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 20 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 19 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 19 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 19 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 18 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 0 }));

        // jump to a different line
        let result = buffer.find(
            "this",
            TextBufferPos { line: 0, byte: 20 },
            SearchDirection::Forward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 2, byte: 4 }));

        let result = buffer.find(
            "this",
            TextBufferPos { line: 2, byte: 3 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 19 }));

        // wrap around
        let result = buffer.find(
            "is",
            TextBufferPos { line: 2, byte: 10 },
            SearchDirection::Forward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 2 }));

        let result = buffer.find(
            "is",
            TextBufferPos { line: 0, byte: 1 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 2, byte: 9 }));
    }
}
