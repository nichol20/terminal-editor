#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]
mod editor;
mod terminal;
use crate::editor::Editor;

fn main() {
    Editor::default().run();
}
