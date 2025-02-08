use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal,
};
use knap_base::math::Vec2f;

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

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;

    queue!(io::stdout(), terminal::EnterAlternateScreen)?;
    queue!(io::stdout(), terminal::DisableLineWrap)?;
    io::stdout().flush()?;

    Ok(())
}

pub fn end_terminal() -> Result<()> {
    queue!(io::stdout(), terminal::EnableLineWrap)?;
    queue!(io::stdout(), terminal::LeaveAlternateScreen)?;
    io::stdout().flush()?;

    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn start_draw() -> Result<TerminalRestoreState> {
    queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    hide_cursor()?;

    Ok(TerminalRestoreState {
        cursor_pos: get_cursor_pos()?,
    })
}

pub fn end_draw(restore_state: &TerminalRestoreState) -> Result<()> {
    move_cursor(restore_state.cursor_pos)?;
    show_cursor()?;

    io::stdout().flush()?;
    Ok(())
}

pub fn size_f64() -> Result<Vec2f> {
    let size = terminal::size()?;
    Ok(Vec2f {
        x: size.0.into(),
        y: size.1.into(),
    })
}

fn hide_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Hide)?;
    Ok(())
}

fn show_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Show)?;
    Ok(())
}

fn move_cursor(pos: TerminalPos) -> Result<()> {
    queue!(io::stdout(), cursor::MoveTo(pos.x, pos.y))?;
    Ok(())
}

pub(crate) fn draw_text<T: AsRef<str>>(pos: TerminalPos, text: T) -> Result<()> {
    move_cursor(pos)?;
    queue!(io::stdout(), style::Print(text.as_ref()))?;
    Ok(())
}

pub(crate) fn draw_colored_text<T: AsRef<str>>(
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

fn get_cursor_pos() -> Result<TerminalPos> {
    let pos = cursor::position()?;
    Ok(TerminalPos { x: pos.0, y: pos.1 })
}

pub fn set_title<T: AsRef<str>>(title: T) -> Result<()> {
    queue!(io::stdout(), terminal::SetTitle(title.as_ref()))?;
    io::stdout().flush()?;

    Ok(())
}
