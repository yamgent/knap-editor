use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    terminal::{self, TerminalPos},
    view::View,
};

pub struct Editor {
    should_quit: bool,
    cursor_pos: TerminalPos,
    view: View,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            view: View::new(),
            should_quit: false,
            cursor_pos: TerminalPos { x: 0, y: 0 },
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
            self.handle_event(&event)?;
            self.draw()?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            let size = terminal::size()?;

            match (code, modifiers) {
                (&KeyCode::Char('q'), &KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (&KeyCode::Left, _) => {
                    if self.cursor_pos.x > 0 {
                        self.cursor_pos.x -= 1;
                    }
                }
                (&KeyCode::Right, _) => {
                    if self.cursor_pos.x < size.x - 1 {
                        self.cursor_pos.x += 1;
                    }
                }
                (&KeyCode::Up, _) => {
                    if self.cursor_pos.y > 0 {
                        self.cursor_pos.y -= 1;
                    }
                }
                (&KeyCode::Down, _) => {
                    if self.cursor_pos.y < size.y - 1 {
                        self.cursor_pos.y += 1;
                    }
                }
                (&KeyCode::Home, _) => {
                    self.cursor_pos.x = 0;
                }
                (&KeyCode::End, _) => {
                    self.cursor_pos.x = size.x - 1;
                }
                (&KeyCode::PageUp, _) => {
                    self.cursor_pos.y = 0;
                }
                (&KeyCode::PageDown, _) => {
                    self.cursor_pos.y = size.y - 1;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn draw(&self) -> Result<()> {
        let mut state = terminal::start_draw()?;

        self.view.render()?;

        state.cursor_pos = self.cursor_pos;

        terminal::end_draw(&state)?;
        Ok(())
    }
}
