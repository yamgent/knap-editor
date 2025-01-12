use std::io;

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};

pub fn clear_screen() -> Result<()> {
    execute!(io::stdout(), Clear(ClearType::All))?;
    Ok(())
}

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;
    clear_screen()?;
    Ok(())
}

pub fn end_terminal() -> Result<()> {
    clear_screen()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn size() -> Result<(u16, u16)> {
    Ok(terminal::size()?)
}

pub fn move_cursor(pos: (u16, u16)) -> Result<()> {
    execute!(io::stdout(), MoveTo(pos.0, pos.1))?;
    Ok(())
}
