use anyhow::Result;

use crate::{
    buffer::Buffer,
    terminal::{self, TerminalPos, TerminalSize},
};

pub struct View {
    buffer: Buffer,
    size: TerminalSize,
}

impl View {
    pub fn new(size: TerminalSize) -> Self {
        Self {
            buffer: Buffer::new(),
            size,
        }
    }

    pub fn new_with_buffer(buffer: Buffer, size: TerminalSize) -> Self {
        Self { buffer, size }
    }

    pub fn resize(&mut self, size: TerminalSize) {
        self.size = size;
    }

    pub fn render(&self) -> Result<()> {
        self.buffer
            .content
            .iter()
            .take(self.size.y as usize)
            .enumerate()
            .map(|(y, line)| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        // y could not be bigger than size.y, which is u16
                        #[allow(clippy::cast_possible_truncation)]
                        y: y as u16,
                    },
                    line.chars().take(self.size.x as usize).collect::<String>(),
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        (self.buffer.content.len()..(self.size.y as usize))
            .map(|y| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        // y could not be bigger than size.y, which is u16
                        #[allow(clippy::cast_possible_truncation)]
                        y: y as u16,
                    },
                    "~",
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(())
    }
}
