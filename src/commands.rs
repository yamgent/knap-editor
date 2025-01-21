#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorCommand {
    QuitAll,
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUpOnePage,
    MoveCursorDownOnePage,
    MoveCursorToStartOfLine,
    MoveCursorToEndOfLine,
    InsertCharacter(char),
    InsertNewline,
    EraseCharacterBeforeCursor,
    EraseCharacterAfterCursor,
    WriteBufferToDisk,
}
