use bitbucket_client::models::{ApiAccount, ApiAccountLinks, ApiLink};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Account {
    #[serde(rename = "type")]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct Link {
    href: Option<String>,
    name: Option<String>,
}

impl From<ApiLink> for Link {
    fn from(api_link: ApiLink) -> Self {
        Self {
            href: api_link.href,
            name: api_link.name,
        }
    }
}
