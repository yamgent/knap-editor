use knap_base::math::Bounds2f;
use knap_window::drawer::Drawer;

pub struct MessageBar {
    bounds: Bounds2f,
    message: Option<String>,
}

impl MessageBar {
    pub fn new(bounds: Bounds2f) -> Self {
        Self {
            bounds,
            message: None,
        }
    }

    pub fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
    }

    pub fn set_message<T: AsRef<str>>(&mut self, message: T) {
        self.message = Some(message.as_ref().to_string());
    }

    pub fn render(&self, drawer: &mut Drawer) {
        if self.bounds.size.x * self.bounds.size.y > 0.0 {
            if let Some(message) = &self.message {
                drawer.draw_text(self.bounds.pos, message);
            }
        }
    }
}
