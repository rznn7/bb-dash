use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::bitbucket_client::BitbucketClient;
use crate::bitbucket_repo::BitbucketRepo;

pub mod account_connected;
pub mod current_repo;
pub mod my_pull_requests_tab;

pub trait Component {
    fn init(&mut self, ctx: &ComponentContext);
    fn update(&mut self);
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event_key(&mut self, key_event: KeyEvent);
}

pub struct ComponentContext {
    pub client: BitbucketClient,
    pub repo: BitbucketRepo,
}
