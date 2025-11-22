#![allow(dead_code)]

use crate::bitbucket_service::BitbucketService;

mod bitbucket_client;
mod bitbucket_service;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let pull_requests = BitbucketService::from_env()?
        .get_authored_pull_requests()
        .await?;

    pull_requests.values.iter().for_each(|p| print!("{:#?}", p));

    Ok(())
}
