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
        terminal::start_draw()?;

        terminal::hide_cursor()?;
        self.draw_rows()?;
        self.draw_debug_text()?;
        terminal::move_cursor(TerminalPos { x: 0, y: 0 })?;
        terminal::show_cursor()?;

        terminal::end_draw()?;
        Ok(())
    }

    fn draw_rows(&self) -> Result<()> {
        let size = terminal::size()?;

        (0..size.y)
            .map(|y| -> Result<()> {
                terminal::move_cursor(TerminalPos { x: 0, y })?;
                terminal::draw_text("~")?;
                Ok(())
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(())
    }

    fn draw_debug_text(&self) -> Result<()> {
        terminal::move_cursor(TerminalPos { x: 5, y: 5 })?;
        terminal::draw_text(&self.debug)?;
        Ok(())
    }
}
