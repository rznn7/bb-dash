use ratatui::Frame;
use ratatui::layout::Rect;

use crate::bitbucket_client::BitbucketClient;
use crate::bitbucket_repo::BitbucketRepo;

pub mod account_connected;

pub trait Component {
    fn init(&mut self, ctx: &ComponentContext);
    fn update(&mut self);
    fn render(&mut self, f: &mut Frame, rect: Rect);
}

pub struct ComponentContext {
    pub client: BitbucketClient,
    pub repo: BitbucketRepo,
}
