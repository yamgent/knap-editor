use knap_base::math::Bounds2f;
use knap_window::drawer::Drawer;

pub(crate) struct MessageBar {
    bounds: Bounds2f,
    message: Option<String>,
}

impl MessageBar {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Bounds2f::ZERO,
            message: None,
        }
    }

    pub(crate) fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
    }

    pub(crate) fn set_message<T: AsRef<str>>(&mut self, message: T) {
        self.message = Some(message.as_ref().to_string());
    }

    pub(crate) fn render(&self, drawer: &mut Drawer) {
        if self.bounds.size.x * self.bounds.size.y > 0.0 {
            if let Some(message) = &self.message {
                drawer.draw_text(self.bounds.pos, message);
            }
        }
    }
}
