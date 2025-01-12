use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};

fn clear_screen() -> Result<()> {
    execute!(io::stdout(), Clear(ClearType::All))?;
    Ok(())
}

fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;
    clear_screen()?;
    Ok(())
}

fn end_terminal() -> Result<()> {
    clear_screen()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        init_terminal().expect("able to initialize terminal");

        let repl_result = self.repl();

        end_terminal().expect("able to deinit terminal");
        repl_result.expect("repl has no fatal error");
    }

    fn repl(&mut self) -> Result<()> {
        while !self.should_quit {
            let event = event::read()?;
            self.handle_event(&event);
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            if matches!(
                (code, modifiers),
                (&KeyCode::Char('q'), &KeyModifiers::CONTROL)
            ) {
                self.should_quit = true;
            }
        }
    }
}
