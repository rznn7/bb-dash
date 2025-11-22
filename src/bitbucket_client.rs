use serde::Deserialize;

#[derive(Debug)]
pub struct BitbucketClient {
    http_client: reqwest::Client,
    base_url: String,
    api_token: String,
    username: String,
}

impl BitbucketClient {
    pub fn new(base_url: String, api_token: String, username: String) -> Self {
        BitbucketClient {
            http_client: reqwest::Client::new(),
            base_url,
            api_token,
            username,
        }
    }
    pub async fn get_current_user(&self) -> Result<Account, reqwest::Error> {
        let url = format!("{}/user", &self.base_url);

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<Account>()
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
pub struct Account {
    #[serde(rename = "type")]
    _type: String,
    links: AccountLinks,
    display_name: String,
    pub uuid: String,
}

#[derive(Deserialize, Debug)]
pub struct AccountLinks {
    avatar: Link,
}

#[derive(Deserialize, Debug)]
struct Link {
    href: Option<String>,
    name: Option<String>,
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
    id: i32,
    title: String,
    summary: PullRequestSummary,
    state: PullRequestState,
    author: Account,
    source: PullRequestEndpoint,
    destination: PullRequestEndpoint,
    merge_commit: Option<PullRequestCommit>,
}

#[derive(Deserialize, Debug)]
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
    branch: PullRequestBranch,
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
    name: String,
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
