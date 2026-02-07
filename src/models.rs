use anyhow::{self};
use bitbucket_client::models::{ApiAccount, ApiAccountLinks, ApiLink};
use bitbucket_client::models::{
    ApiPaginatedPullrequests, ApiPullrequest, ApiPullrequestEndpoint, ApiRepository,
    api_pullrequest::State,
};
use serde::Deserialize;
use strum::Display;
use tracing::warn;

#[derive(Debug)]
pub struct Account {
    pub r#type: String,
    pub links: Option<AccountLinks>,
    pub display_name: Option<String>,
    pub uuid: Option<String>,
}

impl From<ApiAccount> for Account {
    fn from(api_account: ApiAccount) -> Self {
        Self {
            r#type: api_account.r#type,
            links: api_account.links.map(Into::into),
            display_name: api_account.display_name,
            uuid: api_account.uuid,
        }
    }
}

#[derive(Debug)]
pub struct AccountLinks {
    avatar: Option<Box<Link>>,
}

impl From<ApiAccountLinks> for AccountLinks {
    fn from(api_account_links: ApiAccountLinks) -> Self {
        Self {
            avatar: api_account_links
                .avatar
                .map(|boxed_api_link| Box::new(Link::from(*boxed_api_link))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: Option<String>,
    pub name: Option<String>,
}

impl From<ApiLink> for Link {
    fn from(api_link: ApiLink) -> Self {
        Self {
            href: api_link.href,
            name: api_link.name,
        }
    }
}

#[derive(Debug)]
pub struct PaginatedPullRequests {
    size: Option<u32>,
    page: Option<u32>,
    pagelen: Option<u32>,
    next: Option<String>,
    previous: Option<String>,
    pub values: Option<Vec<PullRequest>>,
}

impl From<ApiPaginatedPullrequests> for PaginatedPullRequests {
    fn from(api_paginated_pullrequests: ApiPaginatedPullrequests) -> Self {
        Self {
            size: api_paginated_pullrequests.size,
            page: api_paginated_pullrequests.page,
            pagelen: api_paginated_pullrequests.pagelen,
            next: api_paginated_pullrequests.next,
            previous: api_paginated_pullrequests.previous,
            values: api_paginated_pullrequests.values.map(|api_pullrequests| {
                api_pullrequests
                    .into_iter()
                    .map(PullRequest::try_from)
                    .filter_map(|pr| pr.map_err(|e| warn!("skipping PR: {e}")).ok())
                    .collect()
            }),
        }
    }
}

#[derive(Debug)]
pub struct PullRequest {
    pub r#type: String,
    pub id: i32,
    pub title: Option<String>,
    pub summary: Option<PullRequestSummary>,
    pub state: Option<PullRequestState>,
    pub author: Option<Box<Account>>,
    pub source: Option<Box<PullRequestEndpoint>>,
    pub destination: Option<Box<PullRequestEndpoint>>,
    pub merge_commit: Option<PullRequestCommit>,
    pub links: Option<PullRequestLinks>,
}

impl TryFrom<ApiPullrequest> for PullRequest {
    type Error = anyhow::Error;

    fn try_from(api_pullrequest: ApiPullrequest) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type: api_pullrequest.r#type,
            id: api_pullrequest
                .id
                .ok_or_else(|| anyhow::anyhow!("PullRequest missing id"))?,
            title: api_pullrequest.title,
            summary: api_pullrequest
                .summary
                .and_then(|value| serde_json::from_value(value).ok()),
            state: api_pullrequest.state.map(Into::into),
            author: api_pullrequest
                .author
                .map(|boxed_api_author| Box::new(Account::from(*boxed_api_author))),
            source: api_pullrequest.source.map(|boxed_api_pr_endpoint| {
                Box::new(PullRequestEndpoint::from(*boxed_api_pr_endpoint))
            }),
            destination: api_pullrequest
                .destination
                .map(|boxed_destination| Box::new(PullRequestEndpoint::from(*boxed_destination))),
            merge_commit: api_pullrequest
                .merge_commit
                .and_then(|value| serde_json::from_value(value).ok()),
            links: api_pullrequest
                .links
                .and_then(|value| serde_json::from_value(value).ok()),
        })
    }
}

#[derive(Debug, Display)]
pub enum PullRequestState {
    Open,
    Merged,
    Declined,
    Superseded,
}

impl From<State> for PullRequestState {
    fn from(state: State) -> Self {
        match state {
            State::Open => PullRequestState::Open,
            State::Merged => PullRequestState::Merged,
            State::Declined => PullRequestState::Declined,
            State::Superseded => PullRequestState::Superseded,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PullRequestSummary {
    raw: Option<String>,
    markup: Option<String>,
    html: Option<String>,
}

#[derive(Debug)]
pub struct PullRequestEndpoint {
    repository: Option<Box<Repository>>,
    pub branch: Option<PullRequestBranch>,
    commit: Option<PullRequestCommit>,
}

impl From<ApiPullrequestEndpoint> for PullRequestEndpoint {
    fn from(api_pullrequest_endpoint: ApiPullrequestEndpoint) -> Self {
        Self {
            repository: api_pullrequest_endpoint
                .repository
                .map(|boxed_api_repo| Box::new(Repository::from(*boxed_api_repo))),
            branch: api_pullrequest_endpoint
                .branch
                .and_then(|value| serde_json::from_value(value).ok()),
            commit: api_pullrequest_endpoint
                .commit
                .and_then(|value| serde_json::from_value(value).ok()),
        }
    }
}

#[derive(Debug)]
pub struct Repository {
    r#type: String,
    uuid: String,
    full_name: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

impl From<ApiRepository> for Repository {
    fn from(api_repository: ApiRepository) -> Self {
        Self {
            r#type: api_repository.r#type.clone(),
            uuid: api_repository.r#type.clone(),
            full_name: api_repository.full_name,
            name: api_repository.name,
            description: api_repository.description,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PullRequestBranch {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestCommit {
    hash: String,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestLinks {
    #[serde(rename = "self")]
    pub self_link: Link,
    html: Link,
    commits: Link,
    approve: Link,
    diff: Link,
    diffstat: Link,
    comments: Link,
    activity: Link,
    merge: Link,
    decline: Link,
}
