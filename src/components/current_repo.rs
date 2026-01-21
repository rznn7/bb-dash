use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::components::{Component, ComponentContext};

pub struct CurrentRepoComponent {
    repo_slug: Option<String>,
}

impl CurrentRepoComponent {
    pub fn new() -> Self {
        Self { repo_slug: None }
    }
}

impl Component for CurrentRepoComponent {
    fn init(&mut self, ctx: &ComponentContext) {
        self.repo_slug = Some(ctx.repo.slug().to_string());
    }

    fn update(&mut self) {}

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = CurrentRepoWidget {
            repo_slug: self.repo_slug.clone().unwrap_or(String::from("?")),
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
