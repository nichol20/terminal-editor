use std::{
    error::Error,
    fs::{OpenOptions, create_dir_all},
    sync::Mutex,
};

use tracing_subscriber::EnvFilter;

// error!: terminal/file/event operations failed
// warn!: recoverable unusual conditions
// info!: startup, shutdown, file loaded
// debug!: resize and editor state changes
// trace!: individual commands, cursor movements, rendering

pub fn init() -> Result<(), Box<dyn Error + Send + Sync>> {
    create_dir_all("logs")?;

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/terminal_editor.log")?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("terminal_editor=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_target(true)
        .with_writer(Mutex::new(file))
        .try_init()?;

    Ok(())
}
