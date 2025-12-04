#![allow(dead_code)]

use crate::tui::App;

mod bitbucket_client;
mod bitbucket_repo;
mod bitbucket_service;
mod tui;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
