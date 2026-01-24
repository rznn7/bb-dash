use std::sync::Arc;

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{
    bitbucket_repo::BitbucketRepo,
    components::{Component, KeyEventResponse},
};

pub struct CurrentRepoComponent {
    repo_slug: String,
}

impl CurrentRepoComponent {
    pub fn new(bitbucket_repo: Arc<BitbucketRepo>) -> Self {
        Self {
            repo_slug: bitbucket_repo.slug().to_string(),
        }
    }
}

impl Component for CurrentRepoComponent {
    fn init(&mut self) {}

    fn update(&mut self) {}

    fn handle_event_key(&mut self, _key_event: crossterm::event::KeyEvent) -> KeyEventResponse {
        KeyEventResponse::Ignored
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = CurrentRepoWidget {
            repo_slug: self.repo_slug.clone(),
        };
        frame.render_widget(widget, area);
    }
}

pub struct CurrentRepoWidget {
    pub repo_slug: String,
}

impl Widget for CurrentRepoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let formatted = format!("  {}", self.repo_slug);
        Paragraph::new(formatted).render(area, buf);
    }
}
