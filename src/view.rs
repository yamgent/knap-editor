use anyhow::Result;

use crate::{
    buffer::Buffer,
    commands::EditorCommand,
    math::{Pos2u, ToU16Clamp, ToU64, ToUsizeClamp},
    terminal::TerminalPos,
};

pub struct View {
    buffer: Buffer,
    size: Pos2u,

    text_cursor_pos: Pos2u,
    scroll_offset: Pos2u,

    /// When the cursor moves between the lines on the y-axis
    /// without changing the x position, editors tend to remember
    /// the x pos of the starting line, so that when encountering
    /// lines that are shorter than x (which would require the x
    /// position to be changed, as the cursor is no longer on a
    /// valid position), the original x position is not lost when
    /// the cursor then goes to another line that is longer than
    /// x. Otherwise it would be very disorientating. That's the
    /// job of this variable.
    previous_line_cursor_max_x: Option<u64>,
}

impl View {
    pub fn new(size: Pos2u) -> Self {
        Self {
            buffer: Buffer::new(),
            size,
            text_cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
            previous_line_cursor_max_x: None,
        }
    }

    pub fn new_with_buffer(buffer: Buffer, size: Pos2u) -> Self {
        Self {
            buffer,
            size,
            text_cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
            previous_line_cursor_max_x: None,
        }
    }

    fn get_terminal_x_pos_from_text_cursor_x_pos(&self) -> u16 {
        self.buffer
            .get_line_text_width(
                self.text_cursor_pos.y.to_usize_clamp(),
                self.text_cursor_pos.x.to_usize_clamp(),
            )
            .to_u16_clamp()
    }

    pub fn resize(&mut self, size: Pos2u) {
        self.size = size;
        self.adjust_scroll_to_cursor_pos();
    }

    pub fn render(&self) -> Result<TerminalPos> {
        (0..self.size.y)
            .map(|y| {
                self.buffer.render_line(
                    (self.scroll_offset.y.saturating_add(y)).to_usize_clamp(),
                    TerminalPos {
                        x: 0,
                        y: y.to_u16_clamp(),
                    },
                    self.scroll_offset.x..(self.scroll_offset.x.saturating_add(self.size.x)),
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(TerminalPos {
            x: self
                .get_terminal_x_pos_from_text_cursor_x_pos()
                .saturating_sub(self.scroll_offset.x.to_u16_clamp()),
            y: self
                .text_cursor_pos
                .y
                .saturating_sub(self.scroll_offset.y)
                .to_u16_clamp(),
        })
    }

    fn adjust_scroll_to_cursor_pos(&mut self) {
        let terminal_x = self.get_terminal_x_pos_from_text_cursor_x_pos();

        if terminal_x < self.scroll_offset.x.to_u16_clamp() {
            self.scroll_offset.x = terminal_x as u64;
        }

        if self.text_cursor_pos.y < self.scroll_offset.y {
            self.scroll_offset.y = self.text_cursor_pos.y;
        }

        if terminal_x
            >= self
                .scroll_offset
                .x
                .saturating_add(self.size.x)
                .to_u16_clamp()
        {
            self.scroll_offset.x = terminal_x
                .saturating_sub(self.size.x.to_u16_clamp())
                .saturating_add(1) as u64;
        }

        if self.text_cursor_pos.y >= self.scroll_offset.y.saturating_add(self.size.y) {
            self.scroll_offset.y = self
                .text_cursor_pos
                .y
                .saturating_sub(self.size.y)
                .saturating_add(1);
        }
    }

    /// See `Self::previous_line_cursor_max_x` for more details about the purpose
    /// of this function.
    pub fn adjust_cursor_x_on_cursor_y_movement(&mut self) {
        let line_len = self
            .buffer
            .get_line_len(self.text_cursor_pos.y.to_usize_clamp())
            .to_u64();

        if self.text_cursor_pos.x > line_len {
            // x is not on a valid position, move it back
            if self.previous_line_cursor_max_x.is_none() {
                self.previous_line_cursor_max_x = Some(self.text_cursor_pos.x);
            }
            self.text_cursor_pos.x = line_len;
        } else {
            // check to see if we have previous memory of x
            if let Some(previous_x) = self.previous_line_cursor_max_x {
                if previous_x > line_len {
                    // previous entry still too far out...
                    self.text_cursor_pos.x = line_len;
                } else {
                    self.text_cursor_pos.x = previous_x;
                    self.previous_line_cursor_max_x = None;
                }
            }
        }
    }

    pub fn execute_command(&mut self, command: EditorCommand) -> bool {
        match command {
            EditorCommand::MoveCursorUp => {
                self.text_cursor_pos.y = self.text_cursor_pos.y.saturating_sub(1);
                self.adjust_scroll_to_cursor_pos();
                self.adjust_cursor_x_on_cursor_y_movement();
                true
            }
            EditorCommand::MoveCursorDown => {
                self.text_cursor_pos.y = self
                    .text_cursor_pos
                    .y
                    .saturating_add(1)
                    .clamp(0, self.buffer.get_total_lines().to_u64());
                self.adjust_scroll_to_cursor_pos();
                self.adjust_cursor_x_on_cursor_y_movement();
                true
            }
            EditorCommand::MoveCursorLeft => {
                if self.text_cursor_pos.x == 0 {
                    if self.text_cursor_pos.y > 0 {
                        self.text_cursor_pos.y = self.text_cursor_pos.y.saturating_sub(1);
                        self.text_cursor_pos.x = self
                            .buffer
                            .get_line_len(self.text_cursor_pos.y.to_usize_clamp())
                            .to_u64();
                    }
                } else {
                    self.text_cursor_pos.x = self.text_cursor_pos.x.saturating_sub(1);
                }

                self.adjust_scroll_to_cursor_pos();
                self.previous_line_cursor_max_x.take();
                true
            }
            EditorCommand::MoveCursorRight => {
                let line_len = self
                    .buffer
                    .get_line_len(self.text_cursor_pos.y.to_usize_clamp())
                    .to_u64();

                if self.text_cursor_pos.x == line_len {
                    if self.text_cursor_pos.y < self.buffer.get_total_lines().to_u64() {
                        self.text_cursor_pos.y = self.text_cursor_pos.y.saturating_add(1);
                        self.text_cursor_pos.x = 0;
                    }
                } else {
                    self.text_cursor_pos.x = self.text_cursor_pos.x.saturating_add(1);
                }

                self.adjust_scroll_to_cursor_pos();
                self.previous_line_cursor_max_x.take();
                true
            }
            EditorCommand::MoveCursorUpOnePage => {
                self.text_cursor_pos.y = self.text_cursor_pos.y.saturating_sub(self.size.y);
                self.adjust_scroll_to_cursor_pos();
                self.adjust_cursor_x_on_cursor_y_movement();
                true
            }
            EditorCommand::MoveCursorDownOnePage => {
                self.text_cursor_pos.y = self
                    .text_cursor_pos
                    .y
                    .saturating_add(self.size.y)
                    .clamp(0, self.buffer.get_total_lines().to_u64());
                self.adjust_scroll_to_cursor_pos();
                self.adjust_cursor_x_on_cursor_y_movement();
                true
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.text_cursor_pos.x = 0;
                self.adjust_scroll_to_cursor_pos();
                self.previous_line_cursor_max_x.take();
                true
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.text_cursor_pos.x = self
                    .buffer
                    .get_line_len(self.text_cursor_pos.y.to_usize_clamp())
                    .to_u64();
                self.adjust_scroll_to_cursor_pos();
                self.previous_line_cursor_max_x.take();
                true
            }
            EditorCommand::QuitAll => false,
        }
    }
}
