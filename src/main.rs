#![allow(dead_code)]

use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let bb_client = BitbucketClient {
        http_client: reqwest::Client::new(),
        base_url: std::env::var("BITBUCKET_BASE_URL")?,
        username: std::env::var("BITBUCKET_USERNAME")?,
        api_token: std::env::var("BITBUCKET_API_TOKEN")?,
    };

    let Account {
        uuid: user_uuid, ..
    } = bb_client.get_current_user().await?;

    let PaginatedWorkspaceMembership {
        values: workspace_memberships,
        ..
    } = bb_client.list_workspaces_for_current_user().await?;

    let workspace_uuid = &workspace_memberships
        .first()
        .ok_or("No workspace found")?
        .workspace
        .uuid;

    let PaginatedPullRequests {
        values: pull_requests,
        ..
    } = bb_client
        .list_workspaces_pull_requests_for_a_user(&user_uuid, workspace_uuid)
        .await?;

    pull_requests.iter().for_each(|p| print!("{:#?}", p));

    Ok(())
}

#[derive(Deserialize, Debug)]
struct Account {
    #[serde(rename = "type")]
    _type: String,
    links: AccountLinks,
    display_name: String,
    uuid: String,
}

#[derive(Deserialize, Debug)]
struct AccountLinks {
    avatar: Link,
}

#[derive(Deserialize, Debug)]
struct Link {
    href: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PaginatedWorkspaceMembership {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    values: Vec<WorkspaceMembership>,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    #[serde(rename = "type")]
    _type: String,
    uuid: String,
    name: String,
    slug: String,
}

#[derive(Deserialize, Debug)]
struct WorkspaceMembership {
    user: Account,
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct PaginatedPullRequests {
    size: i32,
    page: i32,
    pagelen: i32,
    next: Option<i32>,
    previous: Option<i32>,
    values: Vec<PullRequest>,
}

#[derive(Deserialize, Debug)]
struct PullRequest {
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
enum PullRequestState {
    Open,
    Merged,
    Declined,
    Superseded,
}

#[derive(Deserialize, Debug)]
struct PullRequestSummary {
    raw: String,
    markup: String,
    html: String,
}

#[derive(Deserialize, Debug)]
struct PullRequestEndpoint {
    repository: Repository,
    branch: PullRequestBranch,
    commit: Option<PullRequestCommit>,
}

#[derive(Deserialize, Debug)]
struct Repository {
    #[serde(rename = "type")]
    _type: String,
    uuid: String,
    full_name: String,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PullRequestBranch {
    name: String,
}

#[derive(Deserialize, Debug)]
enum MergeStrategy {
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
struct PullRequestCommit {
    hash: String,
}

struct BitbucketClient {
    http_client: reqwest::Client,
    base_url: String,
    api_token: String,
    username: String,
}

impl BitbucketClient {
    async fn get_current_user(&self) -> Result<Account, reqwest::Error> {
        let url = format!("{}/user", &self.base_url);

        self.http_client
            .get(url)
            .basic_auth(&self.username, Some(&self.api_token))
            .send()
            .await?
            .json::<Account>()
            .await
    }

    async fn list_workspaces_for_current_user(
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

    async fn list_workspaces_pull_requests_for_a_user(
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
