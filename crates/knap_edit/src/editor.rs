use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use knap_base::math::{Bounds2f, Vec2f};
use knap_window::{drawer::Drawer, window::Window};

use crate::{
    buffer::Buffer,
    command_bar::{CommandBar, CommandBarPrompt},
    commands::EditorCommand,
    message_bar::MessageBar,
    status_bar::StatusBar,
    view::View,
};

pub struct Editor {
    should_quit: bool,
    window: Window,
    drawer: Drawer,

    /// this is used to block the user if he tries to
    /// quit the editor without saving a modified file
    block_quit_remaining_tries: usize,

    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            window: Window::new(),
            drawer: Drawer::new(),
            block_quit_remaining_tries: 0,
            view: View::new(),
            status_bar: StatusBar::new(),
            message_bar: MessageBar::new(),
            command_bar: CommandBar::new(),
        }
    }

    pub fn run(&mut self) {
        self.window.init();
        self.window
            .set_title("[No Name]")
            .expect("able to set title");

        let terminal_size = self.window.size();
        self.handle_new_window_size(terminal_size);
        self.message_bar
            .set_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        self.open_arg_file();

        let repl_result = self.repl();

        self.window.deinit();
        repl_result.expect("repl has no fatal error");
    }

    fn open_arg_file(&mut self) {
        if let Some(filename) = std::env::args().nth(1) {
            let buffer = match Buffer::new_from_file(&filename) {
                Ok(buffer) => buffer,
                Err(err) => {
                    self.message_bar
                        .set_message(format!("Cannot load {filename}: {err}"));
                    return;
                }
            };
            self.view.replace_buffer(buffer, &filename);
            self.window.set_title(&filename).expect("able to set title");
        }
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
        if matches!(command, EditorCommand::QuitAll) {
            if self.block_quit_remaining_tries == 0 {
                self.should_quit = true;
            } else {
                self.message_bar.set_message(format!(
                    "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                    self.block_quit_remaining_tries
                ));
                self.block_quit_remaining_tries = self.block_quit_remaining_tries.saturating_sub(1);
            }
            true
        } else if self.command_bar.has_active_prompt() {
            let result =
                self.command_bar
                    .execute_command(command, &mut self.message_bar, &mut self.view);

            if let Some((prompt, value)) = result.submitted_data {
                self.command_bar.clear_prompt();
                if matches!(prompt, CommandBarPrompt::SaveAs) {
                    self.view.change_filename(&value);
                    self.window.set_title(&value).expect("able to set title");
                    self.execute_command(EditorCommand::WriteBufferToDisk);
                }
            }

            result.is_command_handled
        } else {
            let result =
                self.view
                    .execute_command(command, &mut self.message_bar, &mut self.command_bar);
            self.block_quit_remaining_tries = if self.view.get_status().is_dirty {
                3
            } else {
                0
            };
            result
        }
    }

    fn handle_new_window_size(&mut self, size: Vec2f) {
        self.view.set_bounds(Bounds2f {
            pos: Vec2f::ZERO,
            size: Vec2f {
                x: size.x,
                y: size.y - 2.0,
            },
        });
        self.status_bar.set_bounds(Bounds2f {
            pos: Vec2f {
                x: 0.0,
                y: size.y - 2.0,
            },
            size: Vec2f {
                x: size.x,
                y: if size.y > 1.0 { 1.0 } else { 0.0 },
            },
        });
        self.message_bar.set_bounds(Bounds2f {
            pos: Vec2f {
                x: 0.0,
                y: size.y - 1.0,
            },
            size: Vec2f { x: size.x, y: 1.0 },
        });
        self.command_bar.set_bounds(Bounds2f {
            pos: Vec2f {
                x: 0.0,
                y: size.y - 1.0,
            },
            size: Vec2f { x: size.x, y: 1.0 },
        });
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
                    (&KeyModifiers::NONE | &KeyModifiers::SHIFT, &KeyCode::Char(ch)) => {
                        // NOTE: for SHIFT case, crossterm automatically
                        // converts ch to uppercase for us already. This
                        // also means we do not need to manually handle
                        // capslock scenario
                        Some(EditorCommand::InsertCharacter(ch))
                    }
                    (&KeyModifiers::NONE, &KeyCode::Backspace) => {
                        Some(EditorCommand::EraseCharacterBeforeCursor)
                    }
                    (&KeyModifiers::NONE, &KeyCode::Delete) => {
                        Some(EditorCommand::EraseCharacterAfterCursor)
                    }
                    (&KeyModifiers::NONE, &KeyCode::Tab) => {
                        Some(EditorCommand::InsertCharacter('\t'))
                    }
                    (&KeyModifiers::NONE, &KeyCode::Enter) => Some(EditorCommand::InsertNewline),
                    (&KeyModifiers::CONTROL, &KeyCode::Char('s')) => {
                        Some(EditorCommand::WriteBufferToDisk)
                    }
                    (&KeyModifiers::NONE, &KeyCode::Esc) => Some(EditorCommand::Dismiss),
                    (&KeyModifiers::CONTROL, &KeyCode::Char('f')) => {
                        Some(EditorCommand::StartSearch)
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
                let size = Vec2f {
                    x: f64::from(*width),
                    y: f64::from(*height),
                };

                self.handle_new_window_size(size);
                true
            }
            _ => false,
        }
    }

    fn draw(&mut self) -> Result<()> {
        self.drawer.clear();

        self.view.render(&mut self.drawer);
        self.status_bar
            .render(&mut self.drawer, self.view.get_status());

        if self.command_bar.has_active_prompt() {
            self.command_bar.render(&mut self.drawer);
        } else {
            self.message_bar.render(&mut self.drawer);
        }

        self.drawer.present()?;
        Ok(())
    }
}
