use vello::{
    kurbo::{Affine, Vec2},
    peniko::{color::AlphaColor, Brush},
};

use crate::{
    buffer::FileType,
    drawer::Drawer,
    math::{Bounds2u, Vec2u},
};

pub struct ViewStatus {
    pub filename: Option<String>,
    pub total_lines: usize,
    pub is_dirty: bool,
    pub caret_position: Vec2u,
    pub file_type: FileType,
}

pub struct StatusBar {
    bounds: Bounds2u,
}

impl StatusBar {
    pub fn new(bounds: Bounds2u) -> Self {
        Self { bounds }
    }

    pub fn set_bounds(&mut self, bounds: Bounds2u) {
        self.bounds = bounds;
    }

    pub fn render(&self, drawer: &mut Drawer, view_status: ViewStatus) {
        if self.bounds.size.saturating_area() > 0 {
            let left = format!(
                "{} - {} lines {}",
                view_status.filename.unwrap_or("[No Name]".to_string()),
                view_status.total_lines,
                if view_status.is_dirty {
                    "(modified)"
                } else {
                    "(disk)"
                },
            );

            let right = format!(
                "{} | {}:{}",
                match view_status.file_type {
                    FileType::Rust => "Rust",
                    FileType::PlainText => "Plain Text",
                },
                view_status.caret_position.y.saturating_add(1),
                view_status.caret_position.x.saturating_add(1),
            );

            // TODO: Refactor font size
            // TODO: Also refactor color into theme? Instead of specifying color everywhere
            const FONT_SIZE: f32 = 16.0;

            drawer.draw_rect(
                Brush::Solid(AlphaColor::new([0.7, 0.7, 0.7, 1.0])),
                self.bounds,
            );

            drawer.draw_monospace_text(
                FONT_SIZE,
                Brush::Solid(AlphaColor::BLACK),
                Affine::translate(Vec2::new(
                    self.bounds.pos.x as f64,
                    self.bounds.pos.y as f64,
                )),
                left,
            );

            let right_text_width = drawer.get_monospace_text_width(FONT_SIZE, &right).x;

            drawer.draw_monospace_text(
                FONT_SIZE,
                Brush::Solid(AlphaColor::BLACK),
                Affine::translate(Vec2::new(
                    (self.bounds.pos.x + self.bounds.size.x - right_text_width) as f64,
                    self.bounds.pos.y as f64,
                )),
                right,
            );
        }
    }
}
