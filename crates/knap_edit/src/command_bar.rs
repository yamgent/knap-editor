use knap_base::math::{Bounds2f, Lossy, ToU64, ToUsize, Vec2f, Vec2u};
use knap_window::drawer::Drawer;

use crate::{
    commands::EditorCommand, highlighter::Highlights, message_bar::MessageBar,
    search::SearchDirection, text_line::TextLine, view::View,
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

// TODO: This shares a lot of similar code with the multi-line View,
// explore whether it is possible to share code between this and View
// (or support single-line mode for View and use that).
pub(crate) struct CommandBar {
    bounds: Bounds2f,

    prompt: CommandBarPrompt,
    input: TextLine,

    caret_pos: Vec2u,
    scroll_offset: Vec2u,
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
            input: TextLine::new(""),
            caret_pos: Vec2u::ZERO,
            scroll_offset: Vec2u::ZERO,
        }
    }

    pub(crate) fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
    }

    pub(crate) fn clear_prompt(&mut self) {
        self.prompt = CommandBarPrompt::None;
        self.input = TextLine::new("");
        self.caret_pos = Vec2u::ZERO;
        self.scroll_offset = Vec2u::ZERO;
    }

    pub(crate) fn set_prompt(&mut self, prompt: CommandBarPrompt) {
        self.prompt = prompt;
    }

    pub(crate) fn has_active_prompt(&self) -> bool {
        !matches!(self.prompt, CommandBarPrompt::None)
    }

    pub(crate) fn get_input_bounds(&self) -> Bounds2f {
        let prompt = self.prompt.get_display();
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

            let input_bounds = self.get_input_bounds();

            self.input.render_line(
                drawer,
                Vec2f {
                    x: input_bounds.pos.x,
                    y: self.bounds.pos.y,
                },
                self.scroll_offset.x
                    ..(self
                        .scroll_offset
                        .x
                        .saturating_add(input_bounds.size.x.lossy())),
                &Highlights::new(),
            );

            let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

            let screen_cursor_pos = Vec2u {
                x: <f64 as Lossy<u64>>::lossy(&input_bounds.pos.x)
                    .saturating_add(grid_cursor_pos.x.saturating_sub(self.scroll_offset.x)),
                y: <f64 as Lossy<u64>>::lossy(&input_bounds.pos.y)
                    .saturating_add(grid_cursor_pos.y.saturating_sub(self.scroll_offset.y)),
            };

            drawer.draw_cursor(Vec2f {
                x: screen_cursor_pos.x.lossy(),
                y: screen_cursor_pos.y.lossy(),
            });
        }
    }

    fn get_grid_pos_from_caret_pos(&self, caret_pos: Vec2u) -> Vec2u {
        Vec2u {
            x: self.input.get_line_text_width(caret_pos.x.to_usize()),
            y: caret_pos.y,
        }
    }

    fn adjust_scroll_to_caret_grid_pos(&mut self) {
        let grid_cursor_pos = self.get_grid_pos_from_caret_pos(self.caret_pos);

        if grid_cursor_pos.x < self.scroll_offset.x {
            self.scroll_offset.x = grid_cursor_pos.x;
        }

        let input_bounds = self.get_input_bounds();

        if grid_cursor_pos.x
            >= self
                .scroll_offset
                .x
                .saturating_add(input_bounds.size.x.lossy())
        {
            self.scroll_offset.x = grid_cursor_pos
                .x
                .saturating_sub(input_bounds.size.x.lossy())
                .saturating_add(1);
        }
    }

    fn change_caret_x(&mut self, new_x: u64) {
        self.caret_pos.x = new_x;
        self.adjust_scroll_to_caret_grid_pos();
    }

    fn on_input_updated(&self, view: &mut View) {
        if matches!(self.prompt, CommandBarPrompt::Search) {
            view.find(self.input.to_string(), true, SearchDirection::Forward);
        }
    }

    fn on_find_next(&self, view: &mut View) {
        view.find(self.input.to_string(), false, SearchDirection::Forward);
    }

    fn on_find_previous(&self, view: &mut View) {
        view.find(self.input.to_string(), false, SearchDirection::Backward);
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
                self.change_caret_x(self.caret_pos.x.saturating_sub(1));
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorRight => {
                self.change_caret_x(
                    self.caret_pos
                        .x
                        .saturating_add(1)
                        .clamp(0, self.input.get_line_len().to_u64()),
                );
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.change_caret_x(0);
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.change_caret_x(self.input.get_line_len().to_u64());
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::InsertCharacter(ch) => {
                match self.input.insert_character(self.caret_pos.x.to_usize(), ch) {
                    Ok(result) => {
                        if result.line_len_increased {
                            self.change_caret_x(self.caret_pos.x.saturating_add(1));
                        }
                        self.on_input_updated(view);
                        CommandBarExecuteResult {
                            is_command_handled: true,
                            submitted_data: None,
                        }
                    }
                    Err(..) => CommandBarExecuteResult {
                        is_command_handled: false,
                        submitted_data: None,
                    },
                }
            }
            EditorCommand::InsertNewline => {
                if matches!(self.prompt, CommandBarPrompt::Search) {
                    view.complete_search();
                }

                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: if self.input.get_line_len() > 0 {
                        Some((self.prompt, self.input.to_string()))
                    } else {
                        None
                    },
                }
            }
            EditorCommand::EraseCharacterBeforeCursor => {
                if self.caret_pos.x > 0 {
                    self.input
                        .remove_character(self.caret_pos.x.saturating_sub(1).to_usize());

                    self.change_caret_x(self.caret_pos.x.saturating_sub(1));
                    self.on_input_updated(view);
                }
                CommandBarExecuteResult {
                    is_command_handled: true,
                    submitted_data: None,
                }
            }
            EditorCommand::EraseCharacterAfterCursor => {
                if self.caret_pos.x < self.input.get_line_len().to_u64() {
                    self.input.remove_character(self.caret_pos.x.to_usize());
                    self.on_input_updated(view);
                }
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
