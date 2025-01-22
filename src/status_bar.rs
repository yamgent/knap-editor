use anyhow::Result;

use crate::{
    math::{Pos2u, ToU16Clamp, ToUsizeClamp},
    terminal::{self, TerminalPos},
};

pub struct ViewStatus {
    pub filename: Option<String>,
    pub total_lines: usize,
    pub is_dirty: bool,
    pub caret_position: Pos2u,
}

pub struct StatusBar {
    pos: Pos2u,
    size: Pos2u,
}

impl StatusBar {
    pub fn new(pos: Pos2u, size: Pos2u) -> Self {
        Self { pos, size }
    }

    pub fn reshape(&mut self, pos: Pos2u, size: Pos2u) {
        self.pos = pos;
        self.size = size;
    }

    pub fn render(&self, view_status: ViewStatus) -> Result<()> {
        if self.size.y == 0 {
            return Ok(());
        }

        let size_x = self.size.x.to_usize_clamp();

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
            "{}:{}",
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
                x: self.pos.x.to_u16_clamp(),
                y: self.pos.y.to_u16_clamp(),
            },
            format!(
                "{}{}{}",
                crossterm::style::Attribute::Reverse,
                final_content,
                crossterm::style::Attribute::Reset,
            ),
        )?;

        Ok(())
    }
}
