use crate::loc;
use crate::utils::get_project_dirs;
use anyhow::Result;
use chrono::Local;
use std::fs::File;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

pub fn init_logger() {
    let file_layer = get_and_create_log_path()
        .and_then(|path| File::create(path).map_err(Into::into))
        .map(|file| {
            fmt::layer()
                .with_writer(file)
                .with_ansi(false)
                .with_filter(LevelFilter::INFO)
        })
        .map_err(|e| eprintln!("{}: {:?}", loc::ERR_LOGGER, e))
        .ok();
    let _ =
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(file_layer));
}

fn get_and_create_log_path() -> Result<String> {
    let mut path = get_project_dirs()?.cache_dir().to_owned();
    path.push("logs");
    std::fs::create_dir_all(&path)?;
    path.push(get_log_name());
    Ok(path.to_string_lossy().to_string())
}

fn get_log_name() -> String {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{formatted_time}.log")
}
