#![allow(dead_code)]

use crate::tui::App;
use std::env;

mod bitbucket_api;
mod bitbucket_client;
mod bitbucket_repo;
mod tui;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let repo_path = env::args().nth(1);

    let terminal = ratatui::init();
    let result = App::new(repo_path)?.run(terminal).await;
    ratatui::restore();

    result
}
