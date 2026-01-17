use serde::Deserialize;
use strum::Display;

use crate::models::Account;

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

    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<PaginatedPullRequests, reqwest::Error> {
        let url = format!(
            "{}/repositories/{}/{}/pullrequests",
            &self.base_url, workspace, repo_slug
        );

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<PaginatedPullRequests>()
            .await
    }

    pub async fn list_workspaces_for_current_user(
        &self,
    ) -> Result<PaginatedWorkspaceMembership, reqwest::Error> {
        let url = format!("{}/user/permissions/workspaces", &self.base_url);

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<PaginatedWorkspaceMembership>()
            .await
    }

    pub async fn list_workspaces_pull_requests_for_a_user(
        &self,
        selected_user: &str,
        workspace: &str,
    ) -> Result<PaginatedPullRequests, reqwest::Error> {
        let url = format!(
            "{}/workspaces/{}/pullrequests/{}",
            &self.base_url, workspace, selected_user
        );

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<PaginatedPullRequests>()
            .await
    }
}

#[derive(Deserialize, Debug)]
pub struct PaginatedWorkspaceMembership {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    pub values: Vec<WorkspaceMembership>,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    #[serde(rename = "type")]
    _type: String,
    pub uuid: String,
    name: String,
    slug: String,
}

#[derive(Deserialize, Debug)]
pub struct WorkspaceMembership {
    user: Account,
    pub workspace: Workspace,
}

#[derive(Deserialize, Debug)]
pub struct PaginatedPullRequests {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    pub values: Vec<PullRequest>,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    #[serde(rename = "type")]
    _type: String,
    pub id: i32,
    pub title: String,
    summary: PullRequestSummary,
    pub state: PullRequestState,
    pub author: Account,
    pub source: PullRequestEndpoint,
    pub destination: PullRequestEndpoint,
    merge_commit: Option<PullRequestCommit>,
}

#[derive(Deserialize, Debug, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullRequestState {
    Open,
    Merged,
    Declined,
    Superseded,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestSummary {
    raw: String,
    markup: String,
    html: String,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestEndpoint {
    repository: Repository,
    pub branch: PullRequestBranch,
    commit: Option<PullRequestCommit>,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    #[serde(rename = "type")]
    _type: String,
    uuid: String,
    full_name: String,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestBranch {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub enum MergeStrategy {
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
pub struct PullRequestCommit {
    hash: String,
}
