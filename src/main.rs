#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]
mod buffer;
mod editor;
mod math;
mod terminal;
mod view;

use editor::Editor;

fn main() {
    Editor::new().run();
}
