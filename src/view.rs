use anyhow::Result;

use crate::terminal::{self, TerminalPos};

pub struct View;

impl View {
    pub fn render(&self) -> Result<()> {
        let size = terminal::size()?;

        terminal::draw_text(TerminalPos { x: 0, y: 0 }, "Hello, world")?;
        (1..size.y)
            .map(|y| terminal::draw_text(TerminalPos { x: 0, y }, "~"))
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(())
    }
}
