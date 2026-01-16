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
    let result = match App::new(repo_path) {
        Ok(app) => app.run(terminal).await,
        Err(e) => Err(e),
    };

    ratatui::restore();

    result
}
