#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
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
