use anyhow::Result;

use crate::{
    buffer::Buffer,
    command_bar::{CommandBar, CommandBarPrompt},
    commands::EditorCommand,
    math::{Bounds2u, ToU16Clamp, ToU64, ToUsizeClamp, Vec2u},
    message_bar::MessageBar,
    status_bar::ViewStatus,
    terminal::TerminalPos,
};

pub struct View {
    bounds: Bounds2u,

    buffer: Buffer,

    caret_pos: Vec2u,
    scroll_offset: Vec2u,

    /// When the caret moves between the lines on the y-axis
    /// without changing the x position, editors tend to remember
    /// the x pos of the starting line, so that when encountering
    /// lines that are shorter than x (which would require the x
    /// position to be changed, as the caret is no longer on a
    /// valid position), the original x position is not lost when
    /// the caret then goes to another line that is longer than
    /// x. Otherwise it would be very disorientating. That's the
    /// job of this variable.
    previous_line_caret_max_x: Option<u64>,
}

impl View {
    pub fn new(bounds: Bounds2u) -> Self {
        Self {
            buffer: Buffer::new(),
            bounds,
            caret_pos: Vec2u::ZERO,
            scroll_offset: Vec2u::ZERO,
            previous_line_caret_max_x: None,
        }
    }

    pub fn replace_buffer(&mut self, buffer: Buffer) {
        self.buffer = buffer;
    }

    pub fn change_filename<T: AsRef<str>>(&mut self, filename: T) {
        self.buffer.change_filename(filename);
    }

    pub fn get_status(&self) -> ViewStatus {
        ViewStatus {
            filename: self.buffer.get_filename(),
            total_lines: self.buffer.get_total_lines(),
            is_dirty: self.buffer.get_is_dirty(),
            caret_position: self.caret_pos,
        }
    }

    fn get_grid_pos_from_caret_pos(&self, caret_pos: Vec2u) -> TerminalPos {
        TerminalPos {
            x: self
                .buffer
                .get_line_text_width(caret_pos.y.to_usize_clamp(), caret_pos.x.to_usize_clamp())
                .to_u16_clamp(),
            y: caret_pos.y.to_u16_clamp(),
        }
    }

    pub fn set_bounds(&mut self, bounds: Bounds2u) {
        self.bounds = bounds;
        self.adjust_scroll_to_caret_grid_pos();
    }

    pub fn render(&self) -> Result<TerminalPos> {
        (0..self.bounds.size.y)
            .map(|y| {
                self.buffer.render_line(
                    (self.scroll_offset.y.saturating_add(y)).to_usize_clamp(),
                    TerminalPos {
                        x: self.bounds.pos.x.to_u16_clamp(),
                        y: self
                            .bounds
                            .pos
                            .y
                            .to_u16_clamp()
                            .saturating_add(y.to_u16_clamp()),
                    },
                    self.scroll_offset.x..(self.scroll_offset.x.saturating_add(self.bounds.size.x)),
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

        Ok(TerminalPos {
            x: self.bounds.pos.x.to_u16_clamp().saturating_add(
                grid_cursor_pos
                    .x
                    .saturating_sub(self.scroll_offset.x.to_u16_clamp()),
            ),
            y: self.bounds.pos.y.to_u16_clamp().saturating_add(
                grid_cursor_pos
                    .y
                    .saturating_sub(self.scroll_offset.y.to_u16_clamp()),
            ),
        })
    }

    fn adjust_scroll_to_caret_grid_pos(&mut self) {
        let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

        if grid_cursor_pos.x < self.scroll_offset.x.to_u16_clamp() {
            self.scroll_offset.x = u64::from(grid_cursor_pos.x);
        }

        if grid_cursor_pos.y < self.scroll_offset.y.to_u16_clamp() {
            self.scroll_offset.y = u64::from(grid_cursor_pos.y);
        }

        if grid_cursor_pos.x
            >= self
                .scroll_offset
                .x
                .saturating_add(self.bounds.size.x)
                .to_u16_clamp()
        {
            self.scroll_offset.x = u64::from(
                grid_cursor_pos
                    .x
                    .saturating_sub(self.bounds.size.x.to_u16_clamp())
                    .saturating_add(1),
            );
        }

        if grid_cursor_pos.y
            >= self
                .scroll_offset
                .y
                .saturating_add(self.bounds.size.y)
                .to_u16_clamp()
        {
            self.scroll_offset.y = u64::from(
                grid_cursor_pos
                    .y
                    .saturating_sub(self.bounds.size.y.to_u16_clamp())
                    .saturating_add(1),
            );
        }
    }

    /// See `Self::previous_line_caret_max_x` for more details about the purpose
    /// of this function.
    fn adjust_caret_x_on_caret_y_movement(&mut self) {
        let line_len = self
            .buffer
            .get_line_len(self.caret_pos.y.to_usize_clamp())
            .to_u64();

        if self.caret_pos.x > line_len {
            // x is not on a valid position, move it back
            if self.previous_line_caret_max_x.is_none() {
                self.previous_line_caret_max_x = Some(self.caret_pos.x);
            }
            self.caret_pos.x = line_len;
        } else {
            // check to see if we have previous memory of x
            if let Some(previous_x) = self.previous_line_caret_max_x {
                if previous_x > line_len {
                    // previous entry still too far out...
                    self.caret_pos.x = line_len;
                } else {
                    self.caret_pos.x = previous_x;
                    self.previous_line_caret_max_x = None;
                }
            }
        }
    }

    fn change_caret_x(&mut self, new_x: u64) {
        self.caret_pos.x = new_x;
        self.adjust_scroll_to_caret_grid_pos();
        self.previous_line_caret_max_x.take();
    }

    fn change_caret_y(&mut self, new_y: u64) {
        self.caret_pos.y = new_y;
        self.adjust_caret_x_on_caret_y_movement();
        self.adjust_scroll_to_caret_grid_pos();
    }

    fn change_caret_xy(&mut self, new_pos: Vec2u) {
        self.caret_pos = new_pos;
        self.adjust_scroll_to_caret_grid_pos();
        self.previous_line_caret_max_x.take();
    }

    // splitting the function up doesn't change the readability much
    #[allow(clippy::too_many_lines)]
    pub fn execute_command(
        &mut self,
        command: EditorCommand,
        message_bar: &mut MessageBar,
        command_bar: &mut CommandBar,
    ) -> bool {
        match command {
            EditorCommand::MoveCursorUp => {
                self.change_caret_y(self.caret_pos.y.saturating_sub(1));
                true
            }
            EditorCommand::MoveCursorDown => {
                self.change_caret_y(
                    self.caret_pos
                        .y
                        .saturating_add(1)
                        .clamp(0, self.buffer.get_total_lines().to_u64()),
                );
                true
            }
            EditorCommand::MoveCursorLeft => {
                if self.caret_pos.x == 0 {
                    if self.caret_pos.y > 0 {
                        self.change_caret_xy(Vec2u {
                            x: self
                                .buffer
                                .get_line_len(self.caret_pos.y.saturating_sub(1).to_usize_clamp())
                                .to_u64(),
                            y: self.caret_pos.y.saturating_sub(1),
                        });
                    } else {
                        self.previous_line_caret_max_x.take();
                    }
                } else {
                    self.change_caret_x(self.caret_pos.x.saturating_sub(1));
                }
                true
            }
            EditorCommand::MoveCursorRight => {
                let line_len = self
                    .buffer
                    .get_line_len(self.caret_pos.y.to_usize_clamp())
                    .to_u64();

                if self.caret_pos.x == line_len {
                    if self.caret_pos.y < self.buffer.get_total_lines().to_u64() {
                        self.change_caret_xy(Vec2u {
                            x: 0,
                            y: self.caret_pos.y.saturating_add(1),
                        });
                    } else {
                        self.previous_line_caret_max_x.take();
                    }
                } else {
                    self.change_caret_x(self.caret_pos.x.saturating_add(1));
                }
                true
            }
            EditorCommand::MoveCursorUpOnePage => {
                self.change_caret_y(self.caret_pos.y.saturating_sub(self.bounds.size.y));
                true
            }
            EditorCommand::MoveCursorDownOnePage => {
                self.change_caret_y(
                    self.caret_pos
                        .y
                        .saturating_add(self.bounds.size.y)
                        .clamp(0, self.buffer.get_total_lines().to_u64()),
                );
                true
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.change_caret_x(0);
                true
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.change_caret_x(
                    self.buffer
                        .get_line_len(self.caret_pos.y.to_usize_clamp())
                        .to_u64(),
                );
                true
            }
            EditorCommand::InsertCharacter(ch) => {
                match self.buffer.insert_character(
                    self.caret_pos.y.to_usize_clamp(),
                    self.caret_pos.x.to_usize_clamp(),
                    ch,
                ) {
                    Ok(result) => {
                        if result.line_len_increased {
                            self.change_caret_x(self.caret_pos.x.saturating_add(1));
                        }
                        true
                    }
                    Err(..) => false,
                }
            }
            EditorCommand::EraseCharacterBeforeCursor => {
                if self.caret_pos.x > 0 {
                    self.buffer.remove_character(
                        self.caret_pos.y.to_usize_clamp(),
                        self.caret_pos.x.saturating_sub(1).to_usize_clamp(),
                    );

                    self.change_caret_x(self.caret_pos.x.saturating_sub(1));
                } else if self.caret_pos.y > 0 {
                    let previous_line_len = self
                        .buffer
                        .get_line_len(self.caret_pos.y.saturating_sub(1).to_usize_clamp())
                        .to_u64();

                    self.buffer.join_line_with_below_line(
                        self.caret_pos.y.saturating_sub(1).to_usize_clamp(),
                    );

                    self.change_caret_xy(Vec2u {
                        x: previous_line_len,
                        y: self.caret_pos.y.saturating_sub(1),
                    });
                } else {
                    self.previous_line_caret_max_x.take();
                }
                true
            }
            EditorCommand::EraseCharacterAfterCursor => {
                if self.caret_pos.x
                    < self
                        .buffer
                        .get_line_len(self.caret_pos.y.to_usize_clamp())
                        .to_u64()
                {
                    self.buffer.remove_character(
                        self.caret_pos.y.to_usize_clamp(),
                        self.caret_pos.x.to_usize_clamp(),
                    );
                } else if self.caret_pos.y < self.buffer.get_total_lines().to_u64() {
                    self.buffer
                        .join_line_with_below_line(self.caret_pos.y.to_usize_clamp());
                }

                self.previous_line_caret_max_x.take();
                true
            }

            EditorCommand::InsertNewline => {
                self.buffer.insert_newline_at(
                    self.caret_pos.y.to_usize_clamp(),
                    self.caret_pos.x.to_usize_clamp(),
                );
                self.change_caret_xy(Vec2u {
                    x: 0,
                    y: self.caret_pos.y.saturating_add(1),
                });
                true
            }
            EditorCommand::WriteBufferToDisk => {
                if self.buffer.is_untitled_file() {
                    command_bar.set_prompt(CommandBarPrompt::SaveAs);
                } else {
                    match self.buffer.write_to_disk() {
                        Ok(()) => message_bar.set_message("File saved successfully"),
                        Err(err) => message_bar.set_message(format!("Error writing file: {err:?}")),
                    }
                }
                true
            }
            EditorCommand::StartSearch => {
                command_bar.set_prompt(CommandBarPrompt::Search);
                true
            }
            EditorCommand::QuitAll | EditorCommand::Dismiss => false,
        }
    }
}
