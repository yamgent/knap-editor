use std::panic;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    buffer::Buffer,
    commands::EditorCommand,
    math::Pos2u,
    terminal::{self, TerminalPos},
    view::View,
};

fn setup_panic_hook() {
    let current_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // we can't do anything to recover if end_terminal returns an error,
        // so just ignore the Result
        let _ = terminal::end_terminal();
        current_hook(panic_info);
    }));
}

pub struct Editor {
    should_quit: bool,

    view: View,
    status_bar_text: String,
}

impl Editor {
    pub fn new() -> Self {
        let terminal_size = terminal::size_u64().expect("able to get terminal size");

        Self {
            should_quit: false,
            view: View::new(Pos2u {
                x: terminal_size.x,
                y: terminal_size.y.saturating_sub(1),
            }),
            status_bar_text: "Welcome to hecto".to_string(),
        }
    }

    pub fn run(&mut self) {
        setup_panic_hook();

        terminal::init_terminal().expect("able to initialize terminal");

        self.open_arg_file().expect("open file has no fatal error");

        let repl_result = self.repl();

        terminal::end_terminal().expect("able to deinit terminal");
        repl_result.expect("repl has no fatal error");
    }

    fn open_arg_file(&mut self) -> Result<()> {
        let terminal_size = terminal::size_u64()?;

        if let Some(filename) = std::env::args().nth(1) {
            let buffer = match Buffer::new_from_file(&filename) {
                Ok(buffer) => buffer,
                Err(err) => {
                    self.status_bar_text = format!("Cannot load {filename}: {err}");
                    return Ok(());
                }
            };
            self.view = View::new_with_buffer(
                buffer,
                Pos2u {
                    x: terminal_size.x,
                    y: terminal_size.y.saturating_sub(1),
                },
            );
            self.status_bar_text = format!(r#""{filename}" opened"#);
        }

        Ok(())
    }

    fn repl(&mut self) -> Result<()> {
        while !self.should_quit {
            let event = event::read()?;
            self.handle_event(&event);
            self.draw()?;
        }
        Ok(())
    }

    fn execute_command(&mut self, command: EditorCommand) -> bool {
        match command {
            EditorCommand::QuitAll => {
                self.should_quit = true;
                true
            }
            _ => self.view.execute_command(command),
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let command = match (modifiers, code) {
                    (&KeyModifiers::CONTROL, &KeyCode::Char('q')) => Some(EditorCommand::QuitAll),
                    (&KeyModifiers::NONE, &KeyCode::Up) => Some(EditorCommand::MoveCursorUp),
                    (&KeyModifiers::NONE, &KeyCode::Down) => Some(EditorCommand::MoveCursorDown),
                    (&KeyModifiers::NONE, &KeyCode::Left) => Some(EditorCommand::MoveCursorLeft),
                    (&KeyModifiers::NONE, &KeyCode::Right) => Some(EditorCommand::MoveCursorRight),
                    (&KeyModifiers::NONE, &KeyCode::Home) => {
                        Some(EditorCommand::MoveCursorToStartOfLine)
                    }
                    (&KeyModifiers::NONE, &KeyCode::End) => {
                        Some(EditorCommand::MoveCursorToEndOfLine)
                    }
                    (&KeyModifiers::NONE, &KeyCode::PageUp) => {
                        Some(EditorCommand::MoveCursorUpOnePage)
                    }
                    (&KeyModifiers::NONE, &KeyCode::PageDown) => {
                        Some(EditorCommand::MoveCursorDownOnePage)
                    }
                    _ => None,
                };

                if let Some(command) = command {
                    self.execute_command(command)
                } else {
                    false
                }
            }
            Event::Resize(width, height) => {
                self.view.resize(Pos2u {
                    x: (*width).into(),
                    y: height.saturating_sub(1).into(),
                });
                true
            }
            _ => false,
        }
    }

    fn draw(&self) -> Result<()> {
        let mut state = terminal::start_draw()?;

        let new_cursor_pos = self.view.render()?;

        if !self.status_bar_text.is_empty() {
            let size = terminal::size()?;
            terminal::draw_text(
                TerminalPos {
                    x: 0,
                    y: size.y.saturating_sub(1),
                },
                &self.status_bar_text,
            )?;
        }

        state.cursor_pos = new_cursor_pos;

        terminal::end_draw(&state)?;
        Ok(())
    }
}
