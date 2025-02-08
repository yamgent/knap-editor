use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal,
};
use knap_base::math::Vec2u;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub x: u16,
    pub y: u16,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalPos {
    pub x: u16,
    pub y: u16,
}

pub struct TerminalRestoreState {
    pub cursor_pos: TerminalPos,
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;

    queue!(io::stdout(), terminal::EnterAlternateScreen)?;
    queue!(io::stdout(), terminal::DisableLineWrap)?;
    io::stdout().flush()?;

    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn end_terminal() -> Result<()> {
    queue!(io::stdout(), terminal::EnableLineWrap)?;
    queue!(io::stdout(), terminal::LeaveAlternateScreen)?;
    io::stdout().flush()?;

    terminal::disable_raw_mode()?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn start_draw() -> Result<TerminalRestoreState> {
    queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    hide_cursor()?;

    Ok(TerminalRestoreState {
        cursor_pos: get_cursor_pos()?,
    })
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn end_draw(restore_state: &TerminalRestoreState) -> Result<()> {
    move_cursor(restore_state.cursor_pos)?;
    show_cursor()?;

    io::stdout().flush()?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn size_u64() -> Result<Vec2u> {
    let size = terminal::size()?;
    Ok(Vec2u {
        x: size.0.into(),
        y: size.1.into(),
    })
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
fn hide_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Hide)?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
fn show_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Show)?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
fn move_cursor(pos: TerminalPos) -> Result<()> {
    queue!(io::stdout(), cursor::MoveTo(pos.x, pos.y))?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn draw_text<T: AsRef<str>>(pos: TerminalPos, text: T) -> Result<()> {
    move_cursor(pos)?;
    queue!(io::stdout(), style::Print(text.as_ref()))?;
    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn draw_colored_text<T: AsRef<str>>(
    pos: TerminalPos,
    text: T,
    foreground: Option<Color>,
    background: Option<Color>,
) -> Result<()> {
    move_cursor(pos)?;

    if let Some(foreground) = foreground {
        queue!(io::stdout(), style::SetForegroundColor(foreground))?;
    }

    if let Some(background) = background {
        queue!(io::stdout(), style::SetBackgroundColor(background))?;
    }

    queue!(io::stdout(), style::Print(text.as_ref()))?;

    if foreground.is_some() || background.is_some() {
        queue!(io::stdout(), style::ResetColor)?;
    }

    Ok(())
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
fn get_cursor_pos() -> Result<TerminalPos> {
    let pos = cursor::position()?;
    Ok(TerminalPos { x: pos.0, y: pos.1 })
}

/// # Errors
///
/// Returns `Err` if terminal commands fail.
pub fn set_title<T: AsRef<str>>(title: T) -> Result<()> {
    queue!(io::stdout(), terminal::SetTitle(title.as_ref()))?;
    io::stdout().flush()?;

    Ok(())
}
