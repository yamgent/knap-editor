mod buffer;
mod command_bar;
mod commands;
mod editor;
mod highlighter;
mod message_bar;
mod search;
mod status_bar;
mod terminal;
mod text_line;
mod view;

use editor::Editor;

fn main() {
    Editor::new().run();
}
