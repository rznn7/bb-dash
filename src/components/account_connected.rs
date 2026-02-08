use crate::components::{Component, KeyEventResponse};
use crate::models::Account;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

pub struct AccountConnectedComponent {
    account: Account,
}

impl AccountConnectedComponent {
    pub fn new(account: Account) -> Self {
        Self { account }
    }

    pub fn size(&self) -> u16 {
        AccountConnectedWidget {
            account: &self.account,
        }
        .size()
    }
}

impl Component for AccountConnectedComponent {
    fn init(&mut self) {}

    fn update(&mut self) {}

    fn handle_event_key(&mut self, _key_event: KeyEvent) -> KeyEventResponse {
        KeyEventResponse::Ignored
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = AccountConnectedWidget {
            account: &self.account,
        };
        frame.render_widget(widget, area);
    }
}

pub struct AccountConnectedWidget<'a> {
    pub account: &'a Account,
}

impl AccountConnectedWidget<'_> {
    fn formatted_text(&self) -> String {
        let name = self
            .account
            .display_name
            .as_deref()
            .unwrap_or("unknown");

        format!("  {} ", name)
    }

    pub fn size(&self) -> u16 {
        self.formatted_text().chars().count() as u16
    }
}

impl Widget for AccountConnectedWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.formatted_text())
            .style(Style::default())
            .render(area, buf);
    }
}
