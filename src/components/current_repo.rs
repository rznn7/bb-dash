use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::{bitbucket_repo::BitbucketRepo, components::Component};

pub struct CurrentRepoComponent {
    repo_slug: String,
}

impl CurrentRepoComponent {
    pub fn new(bitbucket_repo: BitbucketRepo) -> Self {
        Self {
            repo_slug: bitbucket_repo.slug().to_string(),
        }
    }
}

impl Component for CurrentRepoComponent {
    fn init(&mut self) {}

    fn update(&mut self) {}

    fn handle_event_key(&mut self, key_event: crossterm::event::KeyEvent) {}

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
