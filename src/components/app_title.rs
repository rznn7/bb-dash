use crate::components::{Component, KeyEventResponse};
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::prelude::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

const APP_TITLE_TEXT: &str = "  bb-dash ";

pub struct AppTitleComponent {
    accent_color: Color,
}

impl AppTitleComponent {
    pub fn new(accent_color: Color) -> Self {
        Self { accent_color }
    }

    pub fn size(&self) -> u16 {
        AppTitleWidget.size()
    }
}

impl Component for AppTitleComponent {
    fn init(&mut self) {}

    fn update(&mut self) {}

    fn handle_event_key(&mut self, _key_event: KeyEvent) -> KeyEventResponse {
        KeyEventResponse::Ignored
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = AppTitleWidget;
        frame.render_widget(widget.with_accent_color(self.accent_color), area);
    }
}

pub struct AppTitleWidget;

impl AppTitleWidget {
    pub fn size(&self) -> u16 {
        APP_TITLE_TEXT.chars().count() as u16
    }

    pub fn with_accent_color(self, accent_color: Color) -> StyledAppTitleWidget {
        StyledAppTitleWidget { accent_color }
    }
}

pub struct StyledAppTitleWidget {
    accent_color: Color,
}

impl Widget for StyledAppTitleWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(APP_TITLE_TEXT)
            .style(Style::default().reversed().fg(self.accent_color))
            .render(area, buf);
    }
}
