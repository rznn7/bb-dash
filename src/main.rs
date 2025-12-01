#![allow(dead_code)]

use crate::{bitbucket_service::BitbucketService, tui::App};

mod bitbucket_client;
mod bitbucket_service;
mod tui;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

//let pull_requests = BitbucketService::from_env()?
//    .get_authored_pull_requests()
//    .await?;

//pull_requests.values.iter().for_each(|p| print!("{:#?}", p));

//Ok(())
