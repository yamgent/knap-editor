use std::io::{self, Write};

use anyhow::Result;
use crossterm::{cursor, queue, style, terminal};

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

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;

    start_draw()?;
    end_draw()?;

    Ok(())
}

pub fn end_terminal() -> Result<()> {
    start_draw()?;
    end_draw()?;

    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn start_draw() -> Result<()> {
    queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

pub fn end_draw() -> Result<()> {
    io::stdout().flush()?;
    Ok(())
}

pub fn size() -> Result<TerminalSize> {
    let size = terminal::size()?;
    Ok(TerminalSize {
        x: size.0,
        y: size.1,
    })
}

pub fn hide_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Hide)?;
    Ok(())
}

pub fn show_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Show)?;
    Ok(())
}

pub fn move_cursor(pos: TerminalPos) -> Result<()> {
    queue!(io::stdout(), cursor::MoveTo(pos.x, pos.y))?;
    Ok(())
}

pub fn draw_text<T: AsRef<str>>(text: T) -> Result<()> {
    queue!(io::stdout(), style::Print(text.as_ref()))?;
    Ok(())
}
