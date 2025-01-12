use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::terminal;

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
        terminal::clear_screen()?;

        let size = terminal::size()?;

        (0..size.1)
            .map(|y| -> Result<()> {
                terminal::move_cursor((0, y))?;
                print!("~");
                Ok(())
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        terminal::move_cursor((5, 5))?;
        print!("{}", self.debug);

        terminal::move_cursor((0, 0))?;

        Ok(())
    }
}
