#![warn(clippy::all, clippy::pedantic)]
mod editor;
use crate::editor::Editor;

fn main() {
    Editor::default().run();
}
