#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorCommand {
    QuitAll,
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorToTopOfBuffer,
    MoveCursorToBottomOfBuffer,
    MoveCursorToStartOfLine,
    MoveCursorToEndOfLine,
}
