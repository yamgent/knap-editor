use anyhow::Result;
use knap_base::{
    color::Color,
    math::{Lossy, Vec2f},
};

use crate::terminal::{self, TerminalPos};

pub struct Drawer {
    queue: Vec<DrawCommand>,
}

enum DrawCommand {
    Text {
        pos: Vec2f,
        text: String,
    },
    ColoredText {
        pos: Vec2f,
        text: String,
        foreground: Option<Color>,
        background: Option<Color>,
    },
    Cursor {
        pos: Vec2f,
    },
}

fn convert_vec2f_to_terminal_pos(pos: Vec2f) -> TerminalPos {
    TerminalPos {
        x: pos.x.lossy(),
        y: pos.y.lossy(),
    }
}

fn convert_color_to_crossterm_color(color: Color) -> crossterm::style::Color {
    crossterm::style::Color::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

impl Drawer {
    pub fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn draw_text<T: AsRef<str>>(&mut self, pos: Vec2f, text: T) {
        self.queue.push(DrawCommand::Text {
            pos,
            text: text.as_ref().to_string(),
        });
    }

    pub fn draw_colored_text<T: AsRef<str>>(
        &mut self,
        pos: Vec2f,
        text: T,
        foreground: Option<Color>,
        background: Option<Color>,
    ) {
        self.queue.push(DrawCommand::ColoredText {
            pos,
            text: text.as_ref().to_string(),
            foreground,
            background,
        });
    }

    pub fn draw_cursor(&mut self, pos: Vec2f) {
        self.queue.push(DrawCommand::Cursor { pos });
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn present(&mut self) -> Result<()> {
        terminal::start_draw()?;

        let mut final_cursor_pos = None;

        self.queue
            .drain(..)
            .map(|command| match command {
                DrawCommand::Text { pos, text } => {
                    terminal::draw_text(convert_vec2f_to_terminal_pos(pos), text)
                }
                DrawCommand::ColoredText {
                    pos,
                    text,
                    foreground,
                    background,
                } => terminal::draw_colored_text(
                    convert_vec2f_to_terminal_pos(pos),
                    text,
                    foreground.map(convert_color_to_crossterm_color),
                    background.map(convert_color_to_crossterm_color),
                ),
                DrawCommand::Cursor { pos } => {
                    final_cursor_pos = Some(pos);
                    Ok(())
                }
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        if let Some(final_cursor_pos) = final_cursor_pos {
            terminal::move_cursor(convert_vec2f_to_terminal_pos(final_cursor_pos))?;
            terminal::show_cursor()?;
        }

        terminal::end_draw()?;

        Ok(())
    }
}
