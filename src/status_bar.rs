use anyhow::Result;

use crate::{
    math::{Pos2u, ToU16Clamp},
    terminal::{self, TerminalPos},
};

pub struct ViewStatus {
    pub filename: Option<String>,
    pub total_lines: usize,
    pub is_dirty: bool,
    pub caret_position: Pos2u,
}

pub fn draw_status_bar(pos: Pos2u, view_status: ViewStatus) -> Result<()> {
    terminal::draw_text(
        TerminalPos {
            x: pos.x.to_u16_clamp(),
            y: pos.y.to_u16_clamp(),
        },
        format!(
            "{} - {} lines {} {}:{}",
            view_status.filename.unwrap_or("[No Name]".to_string()),
            view_status.total_lines,
            if view_status.is_dirty {
                "(modified)"
            } else {
                "(disk)"
            },
            view_status.caret_position.y.saturating_add(1),
            view_status.caret_position.x.saturating_add(1),
        ),
    )?;

    Ok(())
}
