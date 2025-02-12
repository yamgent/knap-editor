use knap_base::math::{Bounds2f, Lossy, Vec2f};
use knap_ui::text_box::TextBox;
use knap_window::drawer::Drawer;

use crate::{
    commands::EditorCommand, message_bar::MessageBar, search::SearchDirection, view::View,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandBarPrompt {
    None,
    SaveAs,
    Search,
}

impl CommandBarPrompt {
    fn get_display(self) -> String {
        match self {
            CommandBarPrompt::None => String::new(),
            CommandBarPrompt::SaveAs => "Save As: ".to_string(),
            CommandBarPrompt::Search => "Search (Esc to cancel, Arrows to navigate): ".to_string(),
        }
    }
}

pub(crate) struct CommandBar {
    bounds: Bounds2f,
    prompt: CommandBarPrompt,
    text_box: TextBox,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CommandBarExecuteResult {
    pub is_command_handled: bool,
    pub submitted_data: Option<(CommandBarPrompt, String)>,
}

impl CommandBar {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Bounds2f::ZERO,
            prompt: CommandBarPrompt::None,
            text_box: TextBox::new(),
        }
    }

    pub(crate) fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
        self.text_box.set_bounds(self.calculate_input_bounds());
    }

    pub(crate) fn clear_prompt(&mut self) {
        self.prompt = CommandBarPrompt::None;
        self.text_box.clear();
    }

    pub(crate) fn set_prompt(&mut self, prompt: CommandBarPrompt) {
        self.prompt = prompt;
        self.text_box.set_bounds(self.calculate_input_bounds());
    }

    pub(crate) fn has_active_prompt(&self) -> bool {
        !matches!(self.prompt, CommandBarPrompt::None)
    }

    fn calculate_input_bounds(&self) -> Bounds2f {
        let prompt = self.prompt.get_display();
        // TODO: When migrating to Vello, this will need to be width instead of chars
        let prompt_len = prompt.chars().count().lossy();
        let input_start_x = self.bounds.pos.x + prompt_len;
        let input_size_x = self.bounds.size.x - prompt_len;

        Bounds2f {
            pos: Vec2f {
                x: input_start_x,
                y: self.bounds.pos.y,
            },
            size: Vec2f {
                x: input_size_x,
                y: self.bounds.size.y,
            },
        }
    }

    pub(crate) fn render(&self, drawer: &mut Drawer) {
        if self.bounds.size.x * self.bounds.size.y > 0.0 {
            let prompt = self.prompt.get_display();
            drawer.draw_text(self.bounds.pos, &prompt);

            self.text_box.render(drawer);
        }
    }

    fn on_input_updated(&self, view: &mut View) {
        if matches!(self.prompt, CommandBarPrompt::Search) {
            view.find(
                self.text_box.get_entire_contents_as_string(),
                true,
                SearchDirection::Forward,
            );
        }
    }

    fn on_find_next(&self, view: &mut View) {
        view.find(
            self.text_box.get_entire_contents_as_string(),
            false,
            SearchDirection::Forward,
        );
    }

    fn on_find_previous(&self, view: &mut View) {
        view.find(
            self.text_box.get_entire_contents_as_string(),
            false,
            SearchDirection::Backward,
        );
    }

    // splitting the function up doesn't change the readability much
    #[allow(clippy::too_many_lines)]
    pub(crate) fn execute_command(
        &mut self,
        command: EditorCommand,
        message_bar: &mut MessageBar,
        view: &mut View,
    ) -> CommandBarExecuteResult {
        match command {
            EditorCommand::QuitAll
            | EditorCommand::WriteBufferToDisk
            | EditorCommand::StartSearch => CommandBarExecuteResult {
                is_command_handled: false,
                submitted_data: None,
            },
            EditorCommand::MoveCursorUp => {
                if matches!(self.prompt, CommandBarPrompt::Search) {
                    self.on_find_previous(view);
                }

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorDown => {
                if matches!(self.prompt, CommandBarPrompt::Search) {
                    self.on_find_next(view);
                }

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorUpOnePage | EditorCommand::MoveCursorDownOnePage => {
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorLeft => {
                self.text_box.move_cursor_left();
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorRight => {
                self.text_box.move_cursor_right();
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.text_box.move_cursor_to_start_of_line();
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.text_box.move_cursor_to_end_of_line();
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::InsertCharacter(ch) => {
                if self.text_box.insert_character_at_cursor(ch).is_ok() {
                    self.on_input_updated(view);
                    CommandBarExecuteResult {
                        is_command_handled: true,
                        submitted_data: None,
                    }
                } else {
                    CommandBarExecuteResult {
                        is_command_handled: false,
                        submitted_data: None,
                    }
                }
            }
            EditorCommand::InsertNewline => {
                if matches!(self.prompt, CommandBarPrompt::Search) {
                    view.complete_search();
                }

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: if self.text_box.get_entire_contents_as_string().is_empty() {
                        None
                    } else {
                        Some((self.prompt, self.text_box.get_entire_contents_as_string()))
                    },
                }
            }
            EditorCommand::EraseCharacterBeforeCursor => {
                self.text_box.erase_character_before_cursor();
                self.on_input_updated(view);

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::EraseCharacterAfterCursor => {
                self.text_box.erase_character_after_cursor();
                self.on_input_updated(view);

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::Dismiss => {
                match self.prompt {
                    CommandBarPrompt::SaveAs => {
                        message_bar.set_message("Save aborted");
                    }
                    CommandBarPrompt::Search => {
                        view.abort_search();
                        message_bar.set_message("Search aborted");
                    }
                    CommandBarPrompt::None => {}
                }

                self.clear_prompt();

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
        }
    }
}
