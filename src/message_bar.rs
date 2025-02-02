use vello::{
    kurbo::{Affine, Vec2},
    peniko::{color::AlphaColor, Brush},
};

use crate::{drawer::Drawer, math::Bounds2u};

pub struct MessageBar {
    bounds: Bounds2u,
    message: Option<String>,
}

impl MessageBar {
    pub fn new(bounds: Bounds2u) -> Self {
        Self {
            bounds,
            message: None,
        }
    }

    pub fn set_bounds(&mut self, bounds: Bounds2u) {
        self.bounds = bounds;
    }

    pub fn set_message<T: AsRef<str>>(&mut self, message: T) {
        self.message = Some(message.as_ref().to_string());
    }

    pub fn render(&self, drawer: &mut Drawer) {
        if self.bounds.size.saturating_area() > 0 {
            if let Some(message) = &self.message {
                // TODO: Refactor font size
                // TODO: Also refactor color into theme? Instead of specifying color everywhere
                const FONT_SIZE: f32 = 16.0;

                drawer.draw_monospace_text(
                    FONT_SIZE,
                    Brush::Solid(AlphaColor::new([0.7, 0.7, 0.7, 1.0])),
                    Affine::translate(Vec2::new(
                        self.bounds.pos.x as f64,
                        self.bounds.pos.y as f64,
                    )),
                    message,
                );
            }
        }
    }
}
