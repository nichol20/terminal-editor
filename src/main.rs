#![warn(clippy::all, clippy::pedantic)]
mod editor;
mod terminal;
use crate::editor::Editor;

fn main() {
    Editor::default().run();
}
