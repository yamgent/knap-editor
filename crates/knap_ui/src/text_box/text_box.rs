use std::ops::Range;

use anyhow::Result;
use knap_base::math::{Bounds2f, Lossy, ToU64, ToUsize, Vec2f, Vec2u};
use knap_window::drawer::Drawer;

use super::{InsertCharError, InsertCharResult, TextHighlights, TextLine};

pub struct TextBox {
    bounds: Bounds2f,

    contents: Vec<TextLine>,
    is_dirty: bool,

    /// Best effort single line mode.
    ///
    /// When this is true, the text box will attempt to
    /// ensure that the caret can only be on a single line.
    ///
    /// However, if the TextBox's content is not single line,
    /// the content will remain multi-line, and be rendered as such,
    /// but the caret will still be constrained to a single line.
    single_line_mode: bool,

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

impl TextBox {
    pub fn new() -> Self {
        Self {
            bounds: Bounds2f::ZERO,
            contents: vec![],
            is_dirty: false,
            single_line_mode: false,
            caret_pos: Vec2u::ZERO,
            scroll_offset: Vec2u::ZERO,
            previous_line_caret_max_x: None,
        }
    }

    /// Best effort single line text box.
    ///
    /// See `Self::single_line_mode` for more details regarding
    /// the "best effort" part.
    pub fn new_single_line_text_box() -> Self {
        Self {
            single_line_mode: true,
            ..Self::new()
        }
    }

    pub fn bounds(&self) -> Bounds2f {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
        self.adjust_scroll_to_caret_grid_pos();
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_is_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }

    pub fn get_line_len(&self, line_idx: usize) -> usize {
        self.contents
            .get(line_idx)
            .map_or(0, TextLine::get_line_len)
    }

    fn get_grid_pos_from_caret_pos(&self, caret_pos: Vec2u) -> Vec2u {
        Vec2u {
            x: self
                .contents
                .get(caret_pos.y.to_usize())
                .map_or(0, |line| line.get_line_text_width(caret_pos.x.to_usize())),
            y: caret_pos.y,
        }
    }

    fn adjust_scroll_to_caret_grid_pos(&mut self) {
        let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

        if grid_cursor_pos.x < self.scroll_offset.x {
            self.scroll_offset.x = grid_cursor_pos.x;
        }

        if grid_cursor_pos.y < self.scroll_offset.y {
            self.scroll_offset.y = grid_cursor_pos.y;
        }

        if grid_cursor_pos.x
            >= self
                .scroll_offset
                .x
                .saturating_add(self.bounds.size.x.lossy())
        {
            self.scroll_offset.x = grid_cursor_pos
                .x
                .saturating_sub(self.bounds.size.x.lossy())
                .saturating_add(1);
        }

        if grid_cursor_pos.y
            >= self
                .scroll_offset
                .y
                .saturating_add(self.bounds.size.y.lossy())
        {
            self.scroll_offset.y = grid_cursor_pos
                .y
                .saturating_sub(self.bounds.size.y.lossy())
                .saturating_add(1);
        }
    }

    /// See `Self::previous_line_caret_max_x` for more details about the purpose
    /// of this function.
    fn adjust_caret_x_on_caret_y_movement(&mut self) {
        let line_len = self.get_line_len(self.caret_pos.y.to_usize()).to_u64();

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

    pub fn move_cursor_up(&mut self) {
        if self.single_line_mode {
            self.change_caret_y(0);
        } else {
            self.change_caret_y(self.caret_pos.y.saturating_sub(1));
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.single_line_mode {
            self.change_caret_y(0);
        } else {
            self.change_caret_y(
                self.caret_pos
                    .y
                    .saturating_add(1)
                    .clamp(0, self.get_total_lines().to_u64()),
            );
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.caret_pos.x == 0 {
            if self.caret_pos.y > 0 && !self.single_line_mode {
                self.change_caret_xy(Vec2u {
                    x: self
                        .get_line_len(self.caret_pos.y.saturating_sub(1).to_usize())
                        .to_u64(),
                    y: self.caret_pos.y.saturating_sub(1),
                });
            } else {
                self.previous_line_caret_max_x.take();
            }
        } else {
            self.change_caret_x(self.caret_pos.x.saturating_sub(1));
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.get_line_len(self.caret_pos.y.to_usize()).to_u64();

        if self.caret_pos.x == line_len {
            if self.caret_pos.y < self.get_total_lines().to_u64() && !self.single_line_mode {
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
    }

    pub fn move_cursor_up_one_page(&mut self) {
        if self.single_line_mode {
            self.change_caret_y(0);
        } else {
            self.change_caret_y(self.caret_pos.y.saturating_sub(self.bounds.size.y.lossy()));
        }
    }

    pub fn move_cursor_down_one_page(&mut self) {
        if self.single_line_mode {
            self.change_caret_y(0);
        } else {
            self.change_caret_y(
                self.caret_pos
                    .y
                    .saturating_add(self.bounds.size.y.lossy())
                    .clamp(0, self.get_total_lines().to_u64()),
            );
        }
    }

    pub fn move_cursor_to_start_of_line(&mut self) {
        self.change_caret_x(0);
    }

    pub fn move_cursor_to_end_of_line(&mut self) {
        self.change_caret_x(self.get_line_len(self.caret_pos.y.to_usize()).to_u64());
    }

    pub fn insert_character_at_cursor(
        &mut self,
        ch: char,
    ) -> Result<InsertCharResult, InsertCharError> {
        let result = if self.caret_pos.y == self.contents.len().to_u64() {
            self.contents.push(TextLine::new(ch.to_string()));
            Ok(InsertCharResult {
                line_len_increased: true,
            })
        } else {
            match self.contents.get_mut(self.caret_pos.y.to_usize()) {
                Some(line) => line.insert_character(self.caret_pos.x.to_usize(), ch),
                None => Err(InsertCharError::InvalidPosition),
            }
        };

        if let Ok(success_result) = &result {
            self.is_dirty = true;
            if success_result.line_len_increased {
                self.change_caret_x(self.caret_pos.x.saturating_add(1));
            }
        }

        result
    }

    fn remove_character(&mut self, line_idx: usize, fragment_idx: usize) {
        if let Some(line) = self.contents.get_mut(line_idx) {
            line.remove_character(fragment_idx);
            self.is_dirty = true;
        }
    }

    fn join_line_with_below_line(&mut self, line_idx: usize) {
        let mut new_line_string = None;

        if let Some(first_line) = self.contents.get(line_idx) {
            if let Some(second_line) = self.contents.get(line_idx.saturating_add(1)) {
                let mut final_string = first_line.to_string();
                final_string.push_str(&second_line.to_string());
                new_line_string = Some(final_string);
            }
        }

        if let Some(new_line) = new_line_string {
            *self
                .contents
                .get_mut(line_idx)
                .expect("line_idx to exist as new_line_string contains line_idx") =
                TextLine::new(new_line);
            self.contents.remove(line_idx.saturating_add(1));
            self.is_dirty = true;
        }
    }

    pub fn erase_character_before_cursor(&mut self) {
        if self.caret_pos.x > 0 {
            self.remove_character(
                self.caret_pos.y.to_usize(),
                self.caret_pos.x.saturating_sub(1).to_usize(),
            );
            // TODO: If the line length still stays the same, then should not subtract 1 from caret_pos.x
            self.change_caret_x(self.caret_pos.x.saturating_sub(1));
            self.is_dirty = true;
        } else if self.caret_pos.y > 0 && !self.single_line_mode {
            let previous_line_len = self
                .get_line_len(self.caret_pos.y.saturating_sub(1).to_usize())
                .to_u64();

            self.join_line_with_below_line(self.caret_pos.y.saturating_sub(1).to_usize());

            self.change_caret_xy(Vec2u {
                x: previous_line_len,
                y: self.caret_pos.y.saturating_sub(1),
            });

            self.is_dirty = true;
        } else {
            self.previous_line_caret_max_x.take();
        }
    }

    pub fn erase_character_after_cursor(&mut self) {
        if self.caret_pos.x < self.get_line_len(self.caret_pos.y.to_usize()).to_u64() {
            self.remove_character(self.caret_pos.y.to_usize(), self.caret_pos.x.to_usize());
            self.is_dirty = true;
        } else if self.caret_pos.y < self.get_total_lines().to_u64() && !self.single_line_mode {
            self.join_line_with_below_line(self.caret_pos.y.to_usize());
            self.is_dirty = true;
        }

        self.previous_line_caret_max_x.take();
    }

    pub fn insert_newline_at_cursor(&mut self) {
        if self.single_line_mode {
            return;
        }

        assert!(self.caret_pos.y <= self.get_total_lines().to_u64());

        match self.contents.get_mut(self.caret_pos.y.to_usize()) {
            Some(line) => {
                let new_line = line.split_off(self.caret_pos.x.to_usize());
                self.contents
                    .insert(self.caret_pos.y.saturating_add(1).to_usize(), new_line);
            }
            None => {
                self.contents.push(TextLine::new(""));
            }
        }

        self.change_caret_xy(Vec2u {
            x: 0,
            y: self.caret_pos.y.saturating_add(1),
        });

        self.is_dirty = true;
    }

    pub fn get_entire_contents_as_string(&self) -> String {
        self.contents
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn reset(&mut self) {
        self.contents = vec![];
        self.caret_pos = Vec2u::ZERO;
        self.scroll_offset = Vec2u::ZERO;
        self.is_dirty = false;
    }

    pub fn get_total_lines(&self) -> usize {
        self.contents.len()
    }

    pub fn render_line(
        &self,
        drawer: &mut Drawer,
        line_idx: usize,
        screen_pos: Vec2f,
        text_offset_x: Range<u64>,
        line_highlight: &TextHighlights,
    ) {
        match self.contents.get(line_idx) {
            Some(line) => line.render_line(drawer, screen_pos, text_offset_x, line_highlight),
            None => {
                if !self.single_line_mode {
                    drawer.draw_text(screen_pos, "~");
                }
            }
        }
    }

    pub fn render(&self, drawer: &mut Drawer) {
        if self.bounds.size.x * self.bounds.size.y > 0.0 {
            (0..self.bounds.size.y.lossy()).for_each(|y| {
                let line_idx = self.scroll_offset.y.saturating_add(y).to_usize();
                self.render_line(
                    drawer,
                    line_idx,
                    Vec2f {
                        x: self.bounds.pos.x,
                        y: self.bounds.pos.y + y.lossy(),
                    },
                    self.scroll_offset.x
                        ..(self
                            .scroll_offset
                            .x
                            .saturating_add(self.bounds.size.x.lossy())),
                    &TextHighlights::new(),
                );
            });

            let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

            let screen_cursor_pos = Vec2u {
                x: <f64 as Lossy<u64>>::lossy(&self.bounds.pos.x)
                    .saturating_add(grid_cursor_pos.x.saturating_sub(self.scroll_offset.x)),
                y: <f64 as Lossy<u64>>::lossy(&self.bounds.pos.y)
                    .saturating_add(grid_cursor_pos.y.saturating_sub(self.scroll_offset.y)),
            };

            drawer.draw_cursor(Vec2f {
                x: screen_cursor_pos.x.lossy(),
                y: screen_cursor_pos.y.lossy(),
            });
        }
    }
}
