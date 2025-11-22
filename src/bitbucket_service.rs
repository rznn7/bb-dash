use anyhow::Context;

use crate::bitbucket_client::{
    Account, BitbucketClient, PaginatedPullRequests, PaginatedWorkspaceMembership,
};

pub struct BitbucketService {
    client: BitbucketClient,
}

impl BitbucketService {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();

        Ok(BitbucketService {
            client: BitbucketClient::new(
                std::env::var("BITBUCKET_BASE_URL")?,
                std::env::var("BITBUCKET_API_TOKEN")?,
                std::env::var("BITBUCKET_USERNAME")?,
            ),
        })
    }

    pub async fn get_authored_pull_requests(&self) -> Result<PaginatedPullRequests, anyhow::Error> {
        let Account {
            uuid: user_uuid, ..
        } = self.client.get_current_user().await?;

        let PaginatedWorkspaceMembership {
            values: workspace_memberships,
            ..
        } = self.client.list_workspaces_for_current_user().await?;

        let workspace_uuid = &workspace_memberships
            .first()
            .context("Failed to find the first workspace uuid")?
            .workspace
            .uuid;

        Ok(self
            .client
            .list_workspaces_pull_requests_for_a_user(&user_uuid, workspace_uuid)
            .await?)
    }
}
