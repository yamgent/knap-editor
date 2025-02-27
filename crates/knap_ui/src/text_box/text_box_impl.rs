use std::ops::Range;

use anyhow::Result;
use knap_base::math::{Bounds2f, Lossy, ToU64, ToUsize, Vec2f, Vec2u};
use knap_window::drawer::Drawer;

use crate::text_buffer::{
    InsertCharError, RemoveCharError, SearchDirection, TextBuffer, TextBufferPos,
};

use super::{TextHighlightLine, TextHighlights, text_line::TextLine};

pub struct InsertCharResult {
    /// There could be scenarios where an insertion of
    /// a new character results in grapheme clusters
    /// merging together. In those situations, the line
    /// length would not increase, and this value would
    /// be false. Therefore the caret position should
    /// not change.
    ///
    /// Otherwise, if there's a length increase, then
    /// the caret position should change, and this
    /// value would be true.
    pub line_len_increased: bool,
}

pub struct RemoveCharResult {
    /// There could be scenarios where a removal of
    /// a character only results in the modification
    /// of a grapheme cluster, instead of a complete
    /// removal of the cluster. In those situations,
    /// the line length would not decrease, and this
    /// value would false. Therefore, the caret position
    /// should not change.
    ///
    /// Otherwise, if there's a length decrease, then
    /// the caret position should change, and this
    /// value would be true.
    pub line_len_decreased: bool,
}

pub struct TextBox<B: TextBuffer> {
    bounds: Bounds2f,

    contents: B,
    is_dirty: bool,

    /// Best effort single line mode.
    ///
    /// When this is true, the text box will attempt to
    /// ensure that the caret can only be on a single line.
    ///
    /// However, if the `TextBox`'s content is not single line,
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

    before_search_caret_pos: Option<Vec2u>,
    before_search_scroll_offset: Option<Vec2u>,
}

impl<B: TextBuffer> TextBox<B> {
    pub fn new(buffer: B) -> Self {
        Self {
            bounds: Bounds2f::ZERO,
            contents: buffer,
            is_dirty: false,
            single_line_mode: false,
            caret_pos: Vec2u::ZERO,
            scroll_offset: Vec2u::ZERO,
            previous_line_caret_max_x: None,
            before_search_caret_pos: None,
            before_search_scroll_offset: None,
        }
    }

    /// Best effort single line text box.
    ///
    /// See `Self::single_line_mode` for more details regarding
    /// the "best effort" part.
    pub fn new_single_line_text_box(buffer: B) -> Self {
        Self {
            single_line_mode: true,
            ..Self::new(buffer)
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

    pub fn set_contents<T: AsRef<str>>(&mut self, contents: T) {
        self.contents.set_contents(contents.as_ref());
        self.is_dirty = true;

        self.caret_pos.y = self.caret_pos.y.clamp(0, self.get_total_lines().to_u64());
        self.caret_pos.x = self
            .caret_pos
            .x
            .clamp(0, self.get_line_len(self.caret_pos.y.to_usize()).to_u64());
        self.adjust_scroll_to_caret_grid_pos();
        self.previous_line_caret_max_x.take();
        self.before_search_caret_pos.take();
        self.before_search_scroll_offset.take();
    }

    pub fn caret_pos(&self) -> Vec2u {
        self.caret_pos
    }

    pub fn get_line_len(&self, line_idx: usize) -> usize {
        self.contents
            .line(line_idx)
            // TODO: This is not efficient
            .map_or(0, |line| TextLine::new(line).get_line_len())
    }

    fn get_grid_pos_from_caret_pos(&self, caret_pos: Vec2u) -> Vec2u {
        Vec2u {
            x: self
                .contents
                .line(caret_pos.y.to_usize())
                // TODO: This is not efficient
                .map_or(0, |line| {
                    TextLine::new(line).get_line_text_width(caret_pos.x.to_usize())
                }),
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
        // TODO: This is not efficient
        let target_line_render = if self.caret_pos.y == self.contents.total_lines().to_u64() {
            TextLine::new("")
        } else {
            match self.contents.line(self.caret_pos.y.to_usize()) {
                Some(line) => TextLine::new(line),
                None => return Err(InsertCharError::InvalidLinePosition),
            }
        };

        // TODO: This logic can be further refactored (it is copied and slightly modifiedacross multiple functions)
        let buffer_pos = TextBufferPos {
            line: self.caret_pos.y.to_usize(),
            byte: match target_line_render
                .get_byte_idx_from_fragment_idx(self.caret_pos.x.to_usize())
            {
                Some(column) => column,
                None => return Err(InsertCharError::InvalidBytePosition),
            },
        };

        self.contents.insert_character_at_pos(buffer_pos, ch)?;
        let line_len_increased = TextLine::new(
            self.contents
                .line(self.caret_pos.y.to_usize())
                .expect("line to exist since we just modified it"),
        )
        .get_line_len()
            > target_line_render.get_line_len();

        self.is_dirty = true;
        if line_len_increased {
            self.change_caret_x(self.caret_pos.x.saturating_add(1));
        }

        Ok(InsertCharResult { line_len_increased })
    }

    fn remove_character(
        &mut self,
        line_idx: usize,
        fragment_idx: usize,
    ) -> Result<RemoveCharResult, RemoveCharError> {
        // TODO: This is not efficient
        let target_line_render = match self.contents.line(line_idx) {
            Some(line) => TextLine::new(line),
            None => return Err(RemoveCharError::InvalidLinePosition),
        };

        // TODO: This logic can be further refactored (it is copied and slightly modifiedacross multiple functions)
        let buffer_pos = TextBufferPos {
            line: line_idx,
            byte: match target_line_render.get_byte_idx_from_fragment_idx(fragment_idx) {
                Some(byte) => byte,
                None => return Err(RemoveCharError::InvalidBytePosition),
            },
        };

        self.contents.remove_character_at_pos(buffer_pos)?;
        self.is_dirty = true;

        if let Some(new_line_render) = self.contents.line(line_idx).map(TextLine::new) {
            Ok(RemoveCharResult {
                line_len_decreased: new_line_render.get_line_len()
                    < target_line_render.get_line_len(),
            })
        } else {
            Ok(RemoveCharResult {
                line_len_decreased: true,
            })
        }
    }

    pub fn erase_character_before_cursor(&mut self) -> Result<RemoveCharResult, RemoveCharError> {
        if self.caret_pos.x > 0 {
            let result = self.remove_character(
                self.caret_pos.y.to_usize(),
                self.caret_pos.x.saturating_sub(1).to_usize(),
            );

            if let Ok(result) = &result {
                if result.line_len_decreased {
                    self.change_caret_x(self.caret_pos.x.saturating_sub(1));
                    self.is_dirty = true;
                }
            }

            self.previous_line_caret_max_x.take();
            result
        } else if self.caret_pos.y > 0 && !self.single_line_mode {
            if self.caret_pos.y == self.contents.total_lines().to_u64() {
                // TODO: this part exist because right now our cursor can actually go
                // beyond the last line (in order to allow insert beyond the last line),
                // but that design will no longer make sense when we introduce vim motions.
                // At that point, we should revisit this code, and perhaps remove this
                // special case.
                self.previous_line_caret_max_x.take();
                return Ok(RemoveCharResult {
                    line_len_decreased: false,
                });
            }

            let previous_line_fragments_len = self
                .get_line_len(self.caret_pos.y.saturating_sub(1).to_usize())
                .to_u64();

            if let Some(previous_line_len) = self
                .contents
                .line_len(self.caret_pos.y.saturating_sub(1).to_usize())
            {
                self.contents.remove_character_at_pos(TextBufferPos {
                        line: self.caret_pos.y.saturating_sub(1).to_usize(),
                        byte: previous_line_len,
                    }).expect("previous line should exist, and it is legal to remove the pos right after the last character");

                self.change_caret_xy(Vec2u {
                    x: previous_line_fragments_len,
                    y: self.caret_pos.y.saturating_sub(1),
                });

                self.is_dirty = true;
            }

            self.previous_line_caret_max_x.take();

            Ok(RemoveCharResult {
                // the (formally) active line was completely wiped out
                // as it gets absorbed by the line above it
                line_len_decreased: true,
            })
        } else {
            self.previous_line_caret_max_x.take();

            Ok(RemoveCharResult {
                line_len_decreased: false,
            })
        }
    }

    pub fn erase_character_after_cursor(&mut self) -> Result<RemoveCharResult, RemoveCharError> {
        if self.caret_pos.x < self.get_line_len(self.caret_pos.y.to_usize()).to_u64() {
            let result =
                self.remove_character(self.caret_pos.y.to_usize(), self.caret_pos.x.to_usize());

            if result.is_ok() {
                self.is_dirty = result.is_ok();
                self.previous_line_caret_max_x.take();
            }

            result
        } else if self.caret_pos.y < self.get_total_lines().to_u64() && !self.single_line_mode {
            let line_len = self
                .contents
                .line_len(self.caret_pos.y.to_usize())
                .expect("line should exist");

            if self
                .contents
                .remove_character_at_pos(TextBufferPos {
                    line: self.caret_pos.y.to_usize(),
                    byte: line_len,
                })
                .is_ok()
            {
                self.is_dirty = true;
            }

            self.previous_line_caret_max_x.take();

            Ok(RemoveCharResult {
                // the active line length actually increased / stay the same, it can't decrease
                line_len_decreased: false,
            })
        } else {
            self.previous_line_caret_max_x.take();
            Ok(RemoveCharResult {
                line_len_decreased: false,
            })
        }
    }

    pub fn insert_newline_at_cursor(&mut self) {
        if self.single_line_mode {
            return;
        }

        assert!(self.caret_pos.y <= self.get_total_lines().to_u64());

        // TODO: This is not efficient
        let target_line_render = match self.contents.line(self.caret_pos.y.to_usize()) {
            Some(line) => TextLine::new(line),
            None => TextLine::new(""),
        };

        // TODO: This logic can be further refactored (it is copied and slightly modifiedacross multiple functions)
        let buffer_pos = TextBufferPos {
            line: self.caret_pos.y.to_usize(),
            byte: match target_line_render
                .get_byte_idx_from_fragment_idx(self.caret_pos.x.to_usize())
            {
                Some(byte) => byte,
                None => return,
            },
        };

        let insert_successful = self
            .contents
            .insert_character_at_pos(buffer_pos, '\n')
            .is_ok();

        self.change_caret_xy(Vec2u {
            x: 0,
            y: self.caret_pos.y.saturating_add(1),
        });

        if insert_successful {
            self.is_dirty = true;
        }
    }

    // TODO: When we use a backend text object (like ropey), this method shouldn't be here
    pub fn get_entire_contents_as_string(&self) -> String {
        self.contents.contents()
    }

    // TODO: When we use a backend text object (like ropey), this method shouldn't be here
    pub fn get_raw_line(&self, line_idx: usize) -> Option<String> {
        self.contents.line(line_idx)
    }

    // TODO: When we use a backend text object (like ropey), this method shouldn't be here
    pub fn get_total_lines(&self) -> usize {
        self.contents.total_lines()
    }

    fn find_in_contents<T: AsRef<str>>(
        &self,
        search: T,
        start_pos: Vec2u,
        search_direction: SearchDirection,
    ) -> Option<Vec2u> {
        // TODO: This is not efficient
        let target_line_render = if start_pos.y == self.contents.total_lines().to_u64() {
            TextLine::new("")
        } else {
            match self.contents.line(start_pos.y.to_usize()) {
                Some(line) => TextLine::new(line),
                None => return None,
            }
        };

        // TODO: This logic can be further refactored (it is copied and slightly modifiedacross multiple functions)
        let buffer_pos = TextBufferPos {
            line: start_pos.y.to_usize(),
            byte: target_line_render.get_byte_idx_from_fragment_idx(start_pos.x.to_usize())?,
        };

        self.contents
            .find(search.as_ref(), buffer_pos, search_direction)
            .map(|result| {
                let final_line_render = TextLine::new(
                    self.contents
                        .line(result.line)
                        .expect("result should return a valid line"),
                );
                let final_fragment_idx = final_line_render
                    .get_fragment_idx_from_byte_idx(result.byte)
                    .expect("result should return a valid byte index");

                Vec2u {
                    x: final_fragment_idx.to_u64(),
                    y: result.line.to_u64(),
                }
            })
    }

    pub fn enter_search_mode(&mut self) {
        self.before_search_caret_pos = Some(self.caret_pos);
        self.before_search_scroll_offset = Some(self.scroll_offset);
    }

    pub fn exit_search_mode(&mut self, retain_search_caret_pos: bool) {
        if retain_search_caret_pos {
            self.before_search_caret_pos.take();
            self.before_search_scroll_offset.take();
        } else {
            self.caret_pos = self
                .before_search_caret_pos
                .take()
                .unwrap_or(self.caret_pos);

            self.scroll_offset = self
                .before_search_scroll_offset
                .take()
                .unwrap_or(self.scroll_offset);
        }
    }

    pub fn find<T: AsRef<str>>(
        &mut self,
        search: T,
        first_search: bool,
        search_direction: SearchDirection,
    ) {
        if let Some(caret_pos) = self.find_in_contents(
            &search,
            if first_search {
                self.before_search_caret_pos.unwrap_or(self.caret_pos)
            } else {
                Vec2u {
                    x: match search_direction {
                        SearchDirection::Forward => self
                            .caret_pos
                            .x
                            .saturating_add(search.as_ref().len().to_u64()),
                        SearchDirection::Backward => self.caret_pos.x,
                    },
                    y: self.caret_pos.y,
                }
            },
            search_direction,
        ) {
            self.change_caret_xy(caret_pos);
        } else if let Some(previous_caret_pos) = self.before_search_caret_pos {
            self.change_caret_xy(previous_caret_pos);
        }
    }

    fn render_line(
        &self,
        drawer: &mut Drawer,
        line_idx: usize,
        screen_pos: Vec2f,
        text_offset_x: Range<u64>,
        line_highlight: &TextHighlightLine,
    ) {
        match self.contents.line(line_idx) {
            Some(line) => {
                // TODO: This is not efficient
                let line_render = TextLine::new(line);
                line_render.render_line(drawer, screen_pos, text_offset_x, line_highlight);
            }
            None => {
                if !self.single_line_mode {
                    drawer.draw_text(screen_pos, "~");
                }
            }
        }
    }

    pub fn render(&self, drawer: &mut Drawer, highlights: &TextHighlights) {
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
                    highlights
                        .line_highlight(line_idx)
                        .unwrap_or(&TextHighlightLine::new()),
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
