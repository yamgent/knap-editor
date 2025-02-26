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

#[cfg(test)]
pub mod buffer_tests {
    use super::*;

    fn test_line<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        // normal
        {
            let mut buffer = new_buffer_fn();
            buffer.set_contents("Hello\nWorld!\n\nThe End");

            assert_eq!(buffer.line(0), Some("Hello".to_string()));
            assert_eq!(buffer.line(1), Some("World!".to_string()));
            assert_eq!(buffer.line(2), Some("".to_string()));
            assert_eq!(buffer.line(3), Some("The End".to_string()));
            assert_eq!(buffer.line(4), None);
        }

        // empty
        {
            let buffer = new_buffer_fn();

            assert_eq!(buffer.line(0), None);
        }
    }

    fn test_line_len<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        // normal
        {
            let mut buffer = new_buffer_fn();
            buffer.set_contents("Hello\nWorld!\n\nThe End");

            assert_eq!(buffer.line_len(0), Some(5));
            assert_eq!(buffer.line_len(1), Some(6));
            assert_eq!(buffer.line_len(2), Some(0));
            assert_eq!(buffer.line_len(3), Some(7));
            assert_eq!(buffer.line_len(4), None);
        }

        // empty
        {
            let buffer = new_buffer_fn();

            assert_eq!(buffer.line_len(0), None);
        }
    }

    fn test_total_lines<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        // normal
        {
            let mut buffer = new_buffer_fn();
            buffer.set_contents("Hello\nWorld!\n\nThe End");

            assert_eq!(buffer.total_lines(), 4);
        }

        // empty
        {
            let buffer = new_buffer_fn();

            assert_eq!(buffer.total_lines(), 0);
        }
    }

    fn test_insert_character_at_pos<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        let mut buffer = new_buffer_fn();

        // insert in empty buffer
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 0 }, 'a');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "a");

        // insert in non-empty buffer
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 1 }, 'b');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "ab");

        // insert at the end of the line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 2 }, 'c');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "abc");

        // insert at the front of the line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 0 }, 'd');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "dabc");

        // insert in the middle of the line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 2 }, 'e');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc");
        assert_eq!(buffer.total_lines(), 1);

        // insert newline character, causes a new line to be created
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 5 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc\n");
        assert_eq!(buffer.total_lines(), 2);

        // can insert character in the new line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 0 }, 'f');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc\nf");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 1 }, 'g');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc\nfg");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 1 }, 'h');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc\nfhg");

        let result = buffer.insert_character_at_pos(TextBufferPos { line: 1, byte: 0 }, 'i');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebc\nifhg");

        // can still insert at old line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 5 }, 'j');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebcj\nifhg");

        // fail to insert beyond the end of the line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 7 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "daebcj\nifhg");

        // fail to insert on a non-existent line that is not directly after the last line
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 0 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidLinePosition));
        assert_eq!(buffer.contents(), "daebcj\nifhg");

        // ok to insert at byte 0 after the last line. a newline should also be automatically inserted
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 2, byte: 0 }, 'k');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "daebcj\nifhg\nk");
        assert_eq!(buffer.total_lines(), 3);

        // but if try to insert at byte non-0 after the last line, it will fail
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 3, byte: 1 }, 'z');
        assert_eq!(result, Err(InsertCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "daebcj\nifhg\nk");
        assert_eq!(buffer.total_lines(), 3);

        // insert newline in the middle of a line causes the line to be split
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 0, byte: 2 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "da\nebcj\nifhg\nk");
        assert_eq!(buffer.total_lines(), 4);

        // insert newline at the start of the line spawns a line before
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 2, byte: 0 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "da\nebcj\n\nifhg\nk");
        assert_eq!(buffer.total_lines(), 5);

        // insert newline at the end of the line spawns a line below
        let result = buffer.insert_character_at_pos(TextBufferPos { line: 2, byte: 0 }, '\n');
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "da\nebcj\n\n\nifhg\nk");
        assert_eq!(buffer.total_lines(), 6);
    }

    fn test_remove_character_at_pos<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        let mut buffer = new_buffer_fn();
        buffer.set_contents("Hello\nWorld!\nAnother\nLine");

        // delete first character of line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 2, byte: 0 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "Hello\nWorld!\nnother\nLine");

        // delete last character of line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 2, byte: 5 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "Hello\nWorld!\nnothe\nLine");

        // delete character in the middle of the line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 2, byte: 2 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "Hello\nWorld!\nnohe\nLine");

        // delete newline character, resulting in a join
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 5 });
        assert_eq!(result, Ok(()));
        assert_eq!(buffer.contents(), "HelloWorld!\nnohe\nLine");

        // fail to delete beyond last line of character (including newline)
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 12 });
        assert_eq!(result, Err(RemoveCharError::InvalidBytePosition));
        assert_eq!(buffer.contents(), "HelloWorld!\nnohe\nLine");

        // fail to delete non-existent line
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 3, byte: 0 });
        assert_eq!(result, Err(RemoveCharError::InvalidLinePosition));
        assert_eq!(buffer.contents(), "HelloWorld!\nnohe\nLine");

        // fail to delete in empty buffer
        buffer.set_contents("");
        let result = buffer.remove_character_at_pos(TextBufferPos { line: 0, byte: 0 });
        assert_eq!(result, Err(RemoveCharError::InvalidLinePosition));
        assert_eq!(buffer.contents(), "");
    }

    fn test_find<B, F>(new_buffer_fn: F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        let mut buffer = new_buffer_fn();
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
            TextBufferPos { line: 0, byte: 18 },
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
            TextBufferPos { line: 0, byte: 21 },
            SearchDirection::Backward,
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
        assert_eq!(result, Some(TextBufferPos { line: 0, byte: 0 }));

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
            TextBufferPos { line: 2, byte: 4 },
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
            TextBufferPos { line: 0, byte: 2 },
            SearchDirection::Backward,
        );
        assert_eq!(result, Some(TextBufferPos { line: 2, byte: 9 }));

        // non-existent search
        let result = buffer.find(
            "cannot be found",
            TextBufferPos { line: 0, byte: 0 },
            SearchDirection::Forward,
        );
        assert_eq!(result, None);

        let result = buffer.find(
            "cannot be found",
            TextBufferPos { line: 2, byte: 5 },
            SearchDirection::Backward,
        );
        assert_eq!(result, None);

        // search in empty buffer
        let empty_buffer = new_buffer_fn();
        let result = empty_buffer.find(
            "nothing to search",
            TextBufferPos { line: 0, byte: 0 },
            SearchDirection::Forward,
        );
        assert_eq!(result, None);
        let result = empty_buffer.find(
            "nothing to search",
            TextBufferPos { line: 0, byte: 0 },
            SearchDirection::Backward,
        );
        assert_eq!(result, None);
    }

    pub fn do_standard_text_buffer_tests<B, F>(new_buffer_fn: &F)
    where
        B: TextBuffer,
        F: Fn() -> B,
    {
        test_line(new_buffer_fn);
        test_line_len(new_buffer_fn);
        test_total_lines(new_buffer_fn);
        test_insert_character_at_pos(new_buffer_fn);
        test_remove_character_at_pos(new_buffer_fn);
        test_find(new_buffer_fn);
    }
}
