use bitbucket_client::apis::configuration::Configuration;

use crate::{
    bitbucket_api::{Account, AppBitbucketApi, PaginatedPullRequests},
    bitbucket_repo::BitbucketRepo,
};

#[derive(Clone, Debug)]
pub struct BitbucketClient {
    bitbucket_api: AppBitbucketApi,
    configuration: Configuration,
}

impl BitbucketClient {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();

        let base_url = std::env::var("BITBUCKET_BASE_URL")?;
        let api_token = std::env::var("BITBUCKET_API_TOKEN")?;
        let username = std::env::var("BITBUCKET_USERNAME")?;

        let bitbucket_api = AppBitbucketApi::new(base_url, api_token.clone(), username.clone());
        let mut configuration = Configuration::new();
        configuration.basic_auth = Some((username, Some(api_token)));

        Ok(BitbucketClient {
            bitbucket_api,
            configuration,
        })
    }

    pub async fn get_user(&self) -> Result<Account, anyhow::Error> {
        let user = self.bitbucket_api.get_current_user().await?;
        Ok(user)
    }

    pub async fn list_pull_requests(
        &self,
        bitbucket_repo: &BitbucketRepo,
    ) -> Result<PaginatedPullRequests, anyhow::Error> {
        let workspace = bitbucket_repo.workspace();
        let repo_slug = bitbucket_repo.slug();
        let pull_requests = self
            .bitbucket_api
            .list_pull_requests(workspace, repo_slug)
            .await?;
        Ok(pull_requests)
    }
}
