#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

mod buffer;
mod command_bar;
mod commands;
mod editor;
mod highlighter;
mod math;
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
