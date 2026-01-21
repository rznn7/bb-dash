#![allow(dead_code)]

use ratatui::style::Color;
use tracing::info;

use crate::{components::account_connected, logging::initialize_logging, tui::App};
use std::env;

mod bitbucket_client;
mod bitbucket_repo;
mod components;
mod fetcher;
mod logging;
mod models;
mod tui;
mod widgets;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    initialize_logging()?;

    info!("app started");

    let repo_path = env::args().nth(1).unwrap_or(String::from("."));
    let accent_color = Color::Green;

    let terminal = ratatui::init();
    let result = match App::new(repo_path, accent_color) {
        Ok(app) => app.run(terminal).await,
        Err(e) => Err(e),
    };

    ratatui::restore();

    result
}
