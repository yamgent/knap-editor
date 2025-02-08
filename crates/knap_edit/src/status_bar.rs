use anyhow::Result;
use knap_base::math::{Bounds2u, ToU16Clamp, ToUsizeClamp, Vec2u};
use knap_window::terminal::{self, TerminalPos};

use crate::buffer::FileType;

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

    pub fn render(&self, view_status: ViewStatus) -> Result<()> {
        if self.bounds.size.saturating_area() > 0 {
            let size_x = self.bounds.size.x.to_usize_clamp();

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

            terminal::draw_text(
                TerminalPos {
                    x: self.bounds.pos.x.to_u16_clamp(),
                    y: self.bounds.pos.y.to_u16_clamp(),
                },
                format!(
                    "{}{}{}",
                    crossterm::style::Attribute::Reverse,
                    final_content,
                    crossterm::style::Attribute::Reset,
                ),
            )?;
        }

        Ok(())
    }
}
