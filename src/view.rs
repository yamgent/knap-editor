use anyhow::Result;

use crate::{
    buffer::Buffer,
    commands::EditorCommand,
    math::{Pos2u, ToU16Clamp, ToU64, ToUsizeClamp},
    terminal::{self, TerminalPos},
};

pub struct View {
    buffer: Buffer,
    size: Pos2u,

    cursor_pos: Pos2u,
    scroll_offset: Pos2u,
}

impl View {
    pub fn new(size: Pos2u) -> Self {
        Self {
            buffer: Buffer::new(),
            size,
            cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
        }
    }

    pub fn new_with_buffer(buffer: Buffer, size: Pos2u) -> Self {
        Self {
            buffer,
            size,
            cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
        }
    }

    pub fn resize(&mut self, size: Pos2u) {
        self.size = size;
        self.adjust_scroll_to_cursor_pos();
    }

    pub fn render(&self) -> Result<TerminalPos> {
        self.buffer
            .content
            .iter()
            .skip(self.scroll_offset.y.to_usize_clamp())
            .take(self.size.y.to_usize_clamp())
            .enumerate()
            .map(|(y, line)| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        y: y.to_u16_clamp(),
                    },
                    line.chars()
                        .skip(self.scroll_offset.x.to_usize_clamp())
                        .take(self.size.x.to_usize_clamp())
                        .collect::<String>(),
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        (self.buffer.content.len()..(self.size.y.to_usize_clamp()))
            .map(|y| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        y: y.to_u16_clamp(),
                    },
                    "~",
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(TerminalPos {
            x: (self.cursor_pos.x - self.scroll_offset.x).to_u16_clamp(),
            y: (self.cursor_pos.y - self.scroll_offset.y).to_u16_clamp(),
        })
    }

    fn adjust_scroll_to_cursor_pos(&mut self) {
        if self.cursor_pos.x < self.scroll_offset.x {
            self.scroll_offset.x = self.cursor_pos.x;
        }

        if self.cursor_pos.y < self.scroll_offset.y {
            self.scroll_offset.y = self.cursor_pos.y;
        }

        if self.cursor_pos.x >= self.scroll_offset.x + self.size.x {
            self.scroll_offset.x = self.cursor_pos.x - self.size.x + 1;
        }

        if self.cursor_pos.y >= self.scroll_offset.y + self.size.y {
            self.scroll_offset.y = self.cursor_pos.y - self.size.y + 1;
        }
    }

    pub fn execute_command(&mut self, command: EditorCommand) -> bool {
        match command {
            EditorCommand::MoveCursorUp => {
                if self.cursor_pos.y > 0 {
                    self.cursor_pos.y -= 1;
                }
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorDown => {
                self.cursor_pos.y += 1;
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorLeft => {
                if self.cursor_pos.x > 0 {
                    self.cursor_pos.x -= 1;
                }
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorRight => {
                self.cursor_pos.x += 1;
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorToTopOfBuffer => {
                self.cursor_pos.y = 0;
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorToBottomOfBuffer => {
                self.cursor_pos.y = self.buffer.content.len().to_u64();
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.cursor_pos.x = 0;
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.cursor_pos.x = if let Some(line) =
                    self.buffer.content.get(self.cursor_pos.y.to_usize_clamp())
                {
                    line.chars().count().saturating_sub(1).to_u64()
                } else {
                    0
                };
                self.adjust_scroll_to_cursor_pos();
                true
            }
            EditorCommand::QuitAll => false,
        }
    }
}
