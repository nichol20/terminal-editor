#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
mod buffer;
mod command;
mod editor;
mod logging;
mod terminal;
mod view;

use crate::editor::Editor;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    logging::init()?;

    info!(version = env!("CARGO_PKG_VERSION"), "starting editor");

    Editor::new().run();

    info!("editor stopped");
    Ok(())
}
