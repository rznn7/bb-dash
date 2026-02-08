#![allow(dead_code)]

use ratatui::style::Color;
use ratatui::widgets::Paragraph;
use tracing::info;

use crate::bitbucket_client::BitbucketClient;
use crate::bitbucket_repo::BitbucketRepo;
use crate::models::Account;
use crate::{logging::initialize_logging, tui::App};
use std::env;
use std::sync::Arc;

mod bitbucket_client;
mod bitbucket_repo;
mod components;
mod fetcher;
mod logging;
mod models;
mod tui;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    initialize_logging()?;

    info!("app starting");

    let mut terminal = ratatui::init();
    terminal.draw(|frame| {
        frame.render_widget(Paragraph::new("Loading user..."), frame.area());
    })?;

    let (bitbucket_client, bitbucket_repo, account) = initialize().await?;
    let accent_color = Color::Blue;

    let app = App::new(bitbucket_client, bitbucket_repo, account, accent_color);
    let result = app.run(terminal).await;

    ratatui::restore();

    info!("app closing");

    result
}

async fn initialize() -> Result<(Arc<BitbucketClient>, Arc<BitbucketRepo>, Account), anyhow::Error>
{
    let repo_path = env::args().nth(1).unwrap_or(String::from("."));

    let bitbucket_client = Arc::new(BitbucketClient::from_env()?);
    let bitbucket_repo = Arc::new(BitbucketRepo::new(&repo_path)?);

    let account = bitbucket_client.get_user().await?;

    Ok((bitbucket_client, bitbucket_repo, account))
}
