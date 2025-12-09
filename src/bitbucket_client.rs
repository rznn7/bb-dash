use crate::{
    bitbucket_api::{Account, BitbucketApi, PaginatedPullRequests},
    bitbucket_repo::BitbucketRepo,
};

#[derive(Clone, Debug)]
pub struct BitbucketClient {
    bitbucket_api: BitbucketApi,
}

impl BitbucketClient {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();

        Ok(BitbucketClient {
            bitbucket_api: BitbucketApi::new(
                std::env::var("BITBUCKET_BASE_URL")?,
                std::env::var("BITBUCKET_API_TOKEN")?,
                std::env::var("BITBUCKET_USERNAME")?,
            ),
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
