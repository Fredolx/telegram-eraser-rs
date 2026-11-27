use anyhow::Result;

mod app;
mod auth;
mod chats;
mod config;
mod eraser;
mod loc;
mod log;
mod telegram;
mod types;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    log::init_logger();
    let mut terminal = ratatui::init();
    let result = app::run(&mut terminal).await;
    ratatui::restore();
    result
}
