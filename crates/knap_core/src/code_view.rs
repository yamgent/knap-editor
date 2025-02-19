use std::{fs::File, io::Write};

use anyhow::Result;
use knap_base::math::Bounds2f;
use knap_ui::{
    text_box::TextBox,
    text_buffer::{RopeTextBuffer, SearchDirection},
};
use knap_window::drawer::Drawer;

use crate::{
    command_bar::{CommandBar, CommandBarPrompt},
    commands::EditorCommand,
    highlighter::HighlightInfo,
    message_bar::MessageBar,
    status_bar::ViewStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileType {
    Rust,
    PlainText,
}

fn deduce_filetype<T: AsRef<str>>(filename: T) -> FileType {
    if filename.as_ref().to_lowercase().ends_with(".rs") {
        FileType::Rust
    } else {
        FileType::PlainText
    }
}

pub(crate) struct CodeView {
    bounds: Bounds2f,

    filename: Option<String>,
    file_type: FileType,

    text_box: TextBox<RopeTextBuffer>,

    highlight_info: HighlightInfo<RopeTextBuffer>,
}

impl CodeView {
    pub(crate) fn new() -> Self {
        Self {
            filename: None,
            file_type: FileType::PlainText,
            text_box: TextBox::new(RopeTextBuffer::new()),
            bounds: Bounds2f::ZERO,
            highlight_info: HighlightInfo::new(),
        }
    }

    pub(crate) fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;
        let mut text_box = TextBox::new(RopeTextBuffer::new());
        text_box.set_contents(content);
        text_box.set_is_dirty(false);

        let filename = Some(filename.as_ref().to_string());
        let file_type = deduce_filetype(filename.as_ref().expect("filename is not None"));
        let mut highlight_info = HighlightInfo::new();
        highlight_info.update_file_type(&text_box, file_type);

        Ok(Self {
            filename,
            file_type,
            text_box,
            bounds: Bounds2f::ZERO,
            highlight_info,
        })
    }

    pub(crate) fn change_filename<T: AsRef<str>>(&mut self, filename: T) {
        self.filename = Some(filename.as_ref().to_string());
        self.file_type = deduce_filetype(filename);
        self.highlight_info
            .update_file_type(&self.text_box, self.file_type);
    }

    pub(crate) fn get_status(&self) -> ViewStatus {
        ViewStatus {
            filename: self.filename.clone(),
            total_lines: self.text_box.get_total_lines(),
            is_dirty: self.text_box.is_dirty(),
            file_type: self.file_type,
            caret_position: self.text_box.caret_pos(),
        }
    }

    pub(crate) fn bounds(&self) -> Bounds2f {
        self.bounds
    }

    pub(crate) fn set_bounds(&mut self, bounds: Bounds2f) {
        self.bounds = bounds;
        self.text_box.set_bounds(bounds);
    }

    pub(crate) fn render(&self, drawer: &mut Drawer) {
        self.text_box
            .render(drawer, self.highlight_info.text_highlight());
    }

    fn start_search(&mut self, command_bar: &mut CommandBar) {
        self.text_box.enter_search_mode();
        command_bar.set_prompt(CommandBarPrompt::Search);
    }

    pub(crate) fn abort_search(&mut self) {
        self.text_box.exit_search_mode(false);
        self.highlight_info.clear_search_highlights(&self.text_box);
    }

    pub(crate) fn complete_search(&mut self) {
        self.text_box.exit_search_mode(true);
        self.highlight_info.clear_search_highlights(&self.text_box);
    }

    pub(crate) fn find<T: AsRef<str>>(
        &mut self,
        search: T,
        first_search: bool,
        search_direction: SearchDirection,
    ) {
        self.text_box.find(&search, first_search, search_direction);
        self.highlight_info.regenerate_on_search_change(
            &self.text_box,
            search,
            self.text_box.caret_pos(),
        );
    }

    fn write_to_disk<T: AsRef<str>>(&mut self, filename: T) -> Result<()> {
        let mut file = File::create(filename.as_ref())?;
        writeln!(file, "{}", self.text_box.get_entire_contents_as_string())?;
        self.text_box.set_is_dirty(false);

        Ok(())
    }

    // splitting the function up doesn't change the readability much
    #[allow(clippy::too_many_lines)]
    pub(crate) fn execute_command(
        &mut self,
        command: EditorCommand,
        message_bar: &mut MessageBar,
        command_bar: &mut CommandBar,
    ) -> bool {
        match command {
            EditorCommand::MoveCursorUp => {
                self.text_box.move_cursor_up();
                true
            }
            EditorCommand::MoveCursorDown => {
                self.text_box.move_cursor_down();
                true
            }
            EditorCommand::MoveCursorLeft => {
                self.text_box.move_cursor_left();
                true
            }
            EditorCommand::MoveCursorRight => {
                self.text_box.move_cursor_right();
                true
            }
            EditorCommand::MoveCursorUpOnePage => {
                self.text_box.move_cursor_up_one_page();
                true
            }
            EditorCommand::MoveCursorDownOnePage => {
                self.text_box.move_cursor_down_one_page();
                true
            }
            EditorCommand::MoveCursorToStartOfLine => {
                self.text_box.move_cursor_to_start_of_line();
                true
            }
            EditorCommand::MoveCursorToEndOfLine => {
                self.text_box.move_cursor_to_end_of_line();
                true
            }
            EditorCommand::InsertCharacter(ch) => {
                if self.text_box.insert_character_at_cursor(ch).is_ok() {
                    self.highlight_info
                        .regenerate_on_buffer_change(&self.text_box);
                    true
                } else {
                    false
                }
            }
            EditorCommand::EraseCharacterBeforeCursor => {
                self.text_box.erase_character_before_cursor();
                self.highlight_info
                    .regenerate_on_buffer_change(&self.text_box);
                true
            }
            EditorCommand::EraseCharacterAfterCursor => {
                self.text_box.erase_character_after_cursor();
                self.highlight_info
                    .regenerate_on_buffer_change(&self.text_box);
                true
            }

            EditorCommand::InsertNewline => {
                self.text_box.insert_newline_at_cursor();
                self.highlight_info
                    .regenerate_on_buffer_change(&self.text_box);
                true
            }
            EditorCommand::WriteBufferToDisk => {
                match &self.filename {
                    Some(filename) => match self.write_to_disk(filename.clone()) {
                        Ok(()) => message_bar.set_message("File saved successfully"),
                        Err(err) => message_bar.set_message(format!("Error writing file: {err:?}")),
                    },
                    None => {
                        command_bar.set_prompt(CommandBarPrompt::SaveAs);
                    }
                }
                true
            }
            EditorCommand::StartSearch => {
                self.start_search(command_bar);
                true
            }
            EditorCommand::QuitAll | EditorCommand::Dismiss => false,
        }
    }
}
