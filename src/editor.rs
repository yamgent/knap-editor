use vello::Scene;
use winit::{
    event::{ElementState, KeyEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, ModifiersState, NamedKey},
};

use crate::{
    buffer::Buffer,
    command_bar::{CommandBar, CommandBarPrompt},
    commands::EditorCommand,
    math::{Bounds2u, Vec2u},
    message_bar::MessageBar,
    status_bar::StatusBar,
    view::View,
};

pub struct Editor {
    /// this is used to block the user if he tries to
    /// quit the editor without saving a modified file
    block_quit_remaining_tries: usize,

    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
}

impl Editor {
    pub fn new(window_size: Vec2u) -> Self {
        // TODO: Actual font size
        const FONT_SIZE: u64 = 16;

        let view = View::new(Bounds2u {
            pos: Vec2u { x: 0, y: 0 },
            size: Vec2u {
                x: window_size.x,
                y: window_size.y.saturating_sub(FONT_SIZE.saturating_mul(2)),
            },
        });

        let status_bar = StatusBar::new(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: window_size.y.saturating_sub(FONT_SIZE.saturating_mul(2)),
            },
            size: Vec2u {
                x: window_size.x,
                y: FONT_SIZE,
            },
        });

        let mut message_bar = MessageBar::new(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: window_size.y.saturating_sub(FONT_SIZE),
            },
            size: Vec2u {
                x: window_size.x,
                y: FONT_SIZE,
            },
        });
        message_bar.set_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        let command_bar = CommandBar::new(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: window_size.y.saturating_sub(FONT_SIZE),
            },
            size: Vec2u {
                x: window_size.x,
                y: FONT_SIZE,
            },
        });

        Self {
            block_quit_remaining_tries: 0,
            view,
            status_bar,
            message_bar,
            command_bar,
        }
    }

    pub fn open_arg_file(&mut self) {
        if let Some(filename) = std::env::args().nth(1) {
            let buffer = match Buffer::new_from_file(&filename) {
                Ok(buffer) => buffer,
                Err(err) => {
                    self.message_bar
                        .set_message(format!("Cannot load {filename}: {err}"));
                    return;
                }
            };
            self.view.replace_buffer(buffer);
        }
    }

    fn execute_command(&mut self, command: EditorCommand, event_loop: &ActiveEventLoop) -> bool {
        if matches!(command, EditorCommand::QuitAll) {
            if self.block_quit_remaining_tries == 0 {
                event_loop.exit();
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
                    self.execute_command(EditorCommand::WriteBufferToDisk, event_loop);
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

    pub fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        modifiers: &ModifiersState,
        event_loop: &ActiveEventLoop,
    ) {
        if matches!(event.state, ElementState::Pressed) {
            // we don't have any commands that rely on release for now
            return;
        }

        let command = if modifiers == &ModifiersState::CONTROL {
            if let Key::Character(ch) = &event.logical_key {
                if ch == "q" {
                    Some(EditorCommand::QuitAll)
                } else if ch == "s" {
                    Some(EditorCommand::WriteBufferToDisk)
                } else if ch == "f" {
                    Some(EditorCommand::StartSearch)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            if let Key::Character(ch) = &event.logical_key {
                // TODO: Do we need to care about SHIFT & CAPS LOCK?
                Some(EditorCommand::InsertCharacter(
                    ch.chars().next().expect("one character"),
                ))
            } else {
                match event.logical_key {
                    Key::Named(NamedKey::ArrowUp) => Some(EditorCommand::MoveCursorUp),
                    Key::Named(NamedKey::ArrowDown) => Some(EditorCommand::MoveCursorDown),
                    Key::Named(NamedKey::ArrowLeft) => Some(EditorCommand::MoveCursorLeft),
                    Key::Named(NamedKey::ArrowRight) => Some(EditorCommand::MoveCursorRight),
                    Key::Named(NamedKey::Home) => Some(EditorCommand::MoveCursorToStartOfLine),
                    Key::Named(NamedKey::End) => Some(EditorCommand::MoveCursorToEndOfLine),
                    Key::Named(NamedKey::PageUp) => Some(EditorCommand::MoveCursorUpOnePage),
                    Key::Named(NamedKey::PageDown) => Some(EditorCommand::MoveCursorDownOnePage),
                    Key::Named(NamedKey::Backspace) => {
                        Some(EditorCommand::EraseCharacterBeforeCursor)
                    }
                    Key::Named(NamedKey::Delete) => Some(EditorCommand::EraseCharacterAfterCursor),
                    Key::Named(NamedKey::Tab) => Some(EditorCommand::InsertCharacter('\t')),
                    Key::Named(NamedKey::Enter) => Some(EditorCommand::InsertNewline),
                    Key::Named(NamedKey::Escape) => Some(EditorCommand::Dismiss),
                    _ => None,
                }
            }
        };

        if let Some(command) = command {
            self.execute_command(command, event_loop);
        }
    }

    pub fn resize(&mut self, new_size: Vec2u) {
        // TODO: Actual font size
        const FONT_SIZE: u64 = 16;

        self.view.set_bounds(Bounds2u {
            pos: Vec2u { x: 0, y: 0 },
            size: Vec2u {
                x: new_size.x,
                y: new_size.y.saturating_sub(FONT_SIZE.saturating_mul(2)),
            },
        });
        self.status_bar.set_bounds(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: new_size.y.saturating_sub(FONT_SIZE.saturating_mul(2)),
            },
            size: Vec2u {
                x: new_size.x,
                y: FONT_SIZE,
            },
        });
        self.message_bar.set_bounds(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: new_size.y.saturating_sub(FONT_SIZE),
            },
            size: Vec2u {
                x: new_size.x,
                y: FONT_SIZE,
            },
        });
        self.command_bar.set_bounds(Bounds2u {
            pos: Vec2u {
                x: 0,
                y: new_size.y.saturating_sub(FONT_SIZE),
            },
            size: Vec2u {
                x: new_size.x,
                y: FONT_SIZE,
            },
        });
    }

    pub fn render(&self, scene: &mut Scene) {
        // TODO: Restore implementation
        let _ = scene;
        // TODO: Handle cursor pos
        /*
        self.view.render()?;
        self.status_bar.render(self.view.get_status())?;

        if self.command_bar.has_active_prompt() {
            self.command_bar.render()?;
        } else {
            self.message_bar.render()?;
        }
        */
    }
}
