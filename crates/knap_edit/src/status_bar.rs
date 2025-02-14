use knap_base::math::{Bounds2f, Lossy, Vec2u};
use knap_window::drawer::Drawer;

use crate::view::FileType;

pub(crate) struct ViewStatus {
    pub filename: Option<String>,
    pub total_lines: usize,
    pub is_dirty: bool,
    pub caret_position: Vec2u,
    pub file_type: FileType,
}

pub(crate) struct StatusBar {
    bounds: Bounds2f,
}

impl StatusBar {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Bounds2f::ZERO,
        }
    }

    pub(crate) fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
    }

    pub(crate) fn render(&self, drawer: &mut Drawer, view_status: ViewStatus) {
        if self.bounds.size.x * self.bounds.size.y > 0.0 {
            let size_x = self.bounds.size.x.lossy();

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

            let final_content = if left.len() > size_x {
                format!("{left:.size_x$}")
            } else if left.len().saturating_add(right.len()) > size_x {
                format!("{left:<size_x$}")
            } else {
                let right_space = size_x.saturating_sub(left.len());
                format!("{left}{right:>right_space$}")
            };

            drawer.draw_text(
                self.bounds.pos,
                format!(
                    "{}{}{}",
                    crossterm::style::Attribute::Reverse,
                    final_content,
                    crossterm::style::Attribute::Reset,
                ),
            );
        }
    }
}
