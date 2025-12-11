use serde::Deserialize;
use strum::Display;

#[derive(Clone, Debug)]
pub struct AppBitbucketApi {
    http_client: reqwest::Client,
    base_url: String,
    api_token: String,
    username: String,
}

impl AppBitbucketApi {
    pub fn new(base_url: String, api_token: String, username: String) -> Self {
        AppBitbucketApi {
            http_client: reqwest::Client::new(),
            base_url,
            api_token,
            username,
        }
    }

    pub async fn get_current_user(&self) -> Result<AppAccount, reqwest::Error> {
        let url = format!("{}/user", &self.base_url);

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<AppAccount>()
            .await
    }

    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<AppPaginatedPullRequests, reqwest::Error> {
        let url = format!(
            "{}/repositories/{}/{}/pullrequests",
            &self.base_url, workspace, repo_slug
        );

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<AppPaginatedPullRequests>()
            .await
    }

    pub async fn list_workspaces_for_current_user(
        &self,
    ) -> Result<AppPaginatedWorkspaceMembership, reqwest::Error> {
        let url = format!("{}/user/permissions/workspaces", &self.base_url);

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<AppPaginatedWorkspaceMembership>()
            .await
    }

    pub async fn list_workspaces_pull_requests_for_a_user(
        &self,
        selected_user: &str,
        workspace: &str,
    ) -> Result<AppPaginatedPullRequests, reqwest::Error> {
        let url = format!(
            "{}/workspaces/{}/pullrequests/{}",
            &self.base_url, workspace, selected_user
        );

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<AppPaginatedPullRequests>()
            .await
    }
}

#[derive(Deserialize, Debug)]
pub struct AppAccount {
    #[serde(rename = "type")]
    pub _type: String,
    pub links: AppAccountLinks,
    pub display_name: String,
    pub uuid: String,
}

#[derive(Deserialize, Debug)]
pub struct AppAccountLinks {
    avatar: AppLink,
}

#[derive(Deserialize, Debug)]
struct AppLink {
    href: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AppPaginatedWorkspaceMembership {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    pub values: Vec<AppWorkspaceMembership>,
}

#[derive(Deserialize, Debug)]
pub struct AppWorkspace {
    #[serde(rename = "type")]
    _type: String,
    pub uuid: String,
    name: String,
    slug: String,
}

#[derive(Deserialize, Debug)]
pub struct AppWorkspaceMembership {
    user: AppAccount,
    pub workspace: AppWorkspace,
}

#[derive(Deserialize, Debug)]
pub struct AppPaginatedPullRequests {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    pub values: Vec<AppPullRequest>,
}

#[derive(Deserialize, Debug)]
pub struct AppPullRequest {
    #[serde(rename = "type")]
    _type: String,
    pub id: i32,
    pub title: String,
    summary: AppPullRequestSummary,
    pub state: AppPullRequestState,
    pub author: AppAccount,
    pub source: AppPullRequestEndpoint,
    pub destination: AppPullRequestEndpoint,
    merge_commit: Option<AppPullRequestCommit>,
}

#[derive(Deserialize, Debug, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum AppPullRequestState {
    Open,
    Merged,
    Declined,
    Superseded,
}

#[derive(Deserialize, Debug)]
pub struct AppPullRequestSummary {
    raw: String,
    markup: String,
    html: String,
}

#[derive(Deserialize, Debug)]
pub struct AppPullRequestEndpoint {
    repository: AppRepository,
    pub branch: AppPullRequestBranch,
    commit: Option<AppPullRequestCommit>,
}

#[derive(Deserialize, Debug)]
pub struct AppRepository {
    #[serde(rename = "type")]
    _type: String,
    uuid: String,
    full_name: String,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AppPullRequestBranch {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub enum AppMergeStrategy {
    #[serde(rename = "merge_commit")]
    MergeCommit,
    #[serde(rename = "squash")]
    Squash,
    #[serde(rename = "fast_forward")]
    FastForward,
    #[serde(rename = "squash_fast_forward")]
    SquashFastForward,
    #[serde(rename = "rebase_fast_forward")]
    RebaseFastForward,
    #[serde(rename = "rebase_merge")]
    RebaseMerge,
}

#[derive(Deserialize, Debug)]
pub struct AppPullRequestCommit {
    hash: String,
}
