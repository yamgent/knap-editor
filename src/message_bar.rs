use anyhow::Result;

use crate::{
    math::{Bounds2u, ToU16Clamp},
    terminal::{self, TerminalPos},
};

pub struct MessageBar {
    message: Option<String>,
    bounds: Bounds2u,
}

impl MessageBar {
    pub fn new(bounds: Bounds2u) -> Self {
        Self {
            message: None,
            bounds,
        }
    }

    pub fn set_message<T: AsRef<str>>(&mut self, message: T) {
        self.message = Some(message.as_ref().to_string());
    }

    pub fn render(&self) -> Result<()> {
        if self.bounds.size.saturating_area() > 0 {
            if let Some(message) = &self.message {
                terminal::draw_text(
                    TerminalPos {
                        x: self.bounds.pos.x.to_u16_clamp(),
                        y: self.bounds.pos.y.to_u16_clamp(),
                    },
                    message,
                )?;
            }
        }

        Ok(())
    }
}
