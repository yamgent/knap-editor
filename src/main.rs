#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]
mod editor;
mod terminal;
mod view;

use editor::Editor;

fn main() {
    Editor::new().run();
}
