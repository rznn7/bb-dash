use ratatui::style::Color;
use ratatui::widgets::Paragraph;
use tracing::info;

use crate::bitbucket_client::BitbucketClient;
use crate::bitbucket_repo::BitbucketRepo;
use crate::models::Account;
use crate::{logging::initialize_logging, tui::App};
use std::env;
use std::sync::Arc;

mod auth;
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

    let args: Vec<String> = env::args().skip(1).collect();
    let auth_flag = args.iter().any(|a| a == "--auth");
    let repo_path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| String::from("."));

    if auth_flag {
        return auth::prompt_and_store();
    }

    let mut terminal = ratatui::init();
    terminal.draw(|frame| {
        frame.render_widget(Paragraph::new("Loading user..."), frame.area());
    })?;

    let result = async {
        let (bitbucket_client, bitbucket_repo, account) = initialize(&repo_path).await?;
        let accent_color = Color::Blue;
        let app = App::new(bitbucket_client, bitbucket_repo, account, accent_color);
        app.run(terminal).await
    }
    .await;

    ratatui::restore();

    info!("app closing");

    result
}

fn resolve_client() -> anyhow::Result<BitbucketClient> {
    dotenv::dotenv().ok();

    if let (Ok(username), Ok(token)) = (
        env::var("BITBUCKET_USERNAME"),
        env::var("BITBUCKET_API_TOKEN"),
    ) {
        return Ok(BitbucketClient::new(username, token));
    }

    match auth::load_stored()? {
        Some(c) => Ok(BitbucketClient::new(c.username, c.api_token)),
        None => Err(anyhow::anyhow!(
            "No credentials found. Run `bb-dash --auth` to store them, or set BITBUCKET_USERNAME and BITBUCKET_API_TOKEN."
        )),
    }
}

async fn initialize(
    repo_path: &str,
) -> Result<(Arc<BitbucketClient>, Arc<BitbucketRepo>, Account), anyhow::Error> {
    let bitbucket_client = Arc::new(resolve_client()?);
    let bitbucket_repo = Arc::new(BitbucketRepo::new(repo_path)?);

    let account = bitbucket_client.get_user().await?;

    Ok((bitbucket_client, bitbucket_repo, account))
}
