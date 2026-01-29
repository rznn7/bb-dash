use bitbucket_client::apis::{
    configuration::Configuration,
    pullrequests_api::repositories_workspace_repo_slug_pullrequests_get, users_api::user_get,
};

use crate::{
    bitbucket_repo::BitbucketRepo,
    models::{Account, PaginatedPullRequests},
};

#[derive(Clone, Debug)]
pub struct BitbucketClient {
    configuration: Configuration,
}

impl BitbucketClient {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();

        let username = std::env::var("BITBUCKET_USERNAME")?;
        let api_token = std::env::var("BITBUCKET_API_TOKEN")?;

        let mut configuration = Configuration::new();
        configuration.basic_auth = Some((username, Some(api_token)));

        Ok(BitbucketClient { configuration })
    }

    pub async fn get_user(&self) -> Result<Account, anyhow::Error> {
        let api_account: Account = user_get(&self.configuration).await?.into();
        Ok(api_account)
    }

    pub async fn list_pull_requests(
        &self,
        bitbucket_repo: &BitbucketRepo,
        state: Option<&str>,
        q: Option<&str>,
    ) -> Result<PaginatedPullRequests, anyhow::Error> {
        let workspace = bitbucket_repo.workspace();
        let repo_slug = bitbucket_repo.slug();
        let pull_requests = repositories_workspace_repo_slug_pullrequests_get(
            &self.configuration,
            repo_slug,
            workspace,
            state,
            q,
        )
        .await?
        .into();

        Ok(pull_requests)
    }
}
