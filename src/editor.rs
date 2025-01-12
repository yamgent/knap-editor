use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::terminal::{self, TerminalPos};

pub struct Editor {
    // TODO: Remove debug
    debug: String,

    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            debug: String::new(),
            should_quit: false,
        }
    }

    pub fn run(&mut self) {
        terminal::init_terminal().expect("able to initialize terminal");

        let repl_result = self.repl();

        terminal::end_terminal().expect("able to deinit terminal");
        repl_result.expect("repl has no fatal error");
    }

    fn repl(&mut self) -> Result<()> {
        while !self.should_quit {
            let event = event::read()?;
            self.handle_event(&event);
            self.draw()?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            self.debug = format!("{code:?} {modifiers:?}");

            if matches!(
                (code, modifiers),
                (&KeyCode::Char('q'), &KeyModifiers::CONTROL)
            ) {
                self.should_quit = true;
            }
        }
    }

    fn draw(&self) -> Result<()> {
        let state = terminal::start_draw()?;

        self.draw_row_columns()?;
        self.draw_debug_text()?;

        terminal::end_draw(&state)?;
        Ok(())
    }

    fn draw_row_columns(&self) -> Result<()> {
        let size = terminal::size()?;

        (0..size.y)
            .map(|y| terminal::draw_text(TerminalPos { x: 0, y }, "~"))
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(())
    }

    fn draw_debug_text(&self) -> Result<()> {
        let size = terminal::size()?;

        let x_pos = size.x / 3;
        let y_pos = size.y / 3;

        terminal::draw_text(
            TerminalPos { x: x_pos, y: y_pos },
            "hecto editor -- version 0.1.0",
        )?;
        terminal::draw_text(
            TerminalPos {
                x: x_pos,
                y: y_pos + 1,
            },
            &self.debug,
        )?;
        Ok(())
    }
}
