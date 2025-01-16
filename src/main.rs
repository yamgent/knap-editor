#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::as_conversions
)]

mod buffer;
mod commands;
mod editor;
mod math;
mod terminal;
mod view;

use editor::Editor;

fn main() {
    Editor::new().run();
}
