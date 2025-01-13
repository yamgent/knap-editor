use anyhow::Result;

use crate::{
    buffer::Buffer,
    terminal::{self, TerminalPos},
};

pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
        }
    }

    pub fn render(&self) -> Result<()> {
        let size = terminal::size()?;

        self.buffer
            .content
            .iter()
            .take(size.y as usize)
            .enumerate()
            .map(|(y, line)| terminal::draw_text(TerminalPos { x: 0, y: y as u16 }, line))
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        (self.buffer.content.len()..(size.y as usize))
            .map(|y| terminal::draw_text(TerminalPos { x: 0, y: y as u16 }, "~"))
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(())
    }
}
