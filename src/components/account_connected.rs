use crate::components::Component;
use crate::components::ComponentContext;
use crate::fetcher::{Fetcher, ResourceState};
use crate::models::Account;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

const LOADING_TEXT: &str = "...";

pub struct AccountConnectedComponent {
    account_connected: ResourceState<Account>,
    account_connected_fetcher: Option<Fetcher<Account>>,
}

impl AccountConnectedComponent {
    pub fn new() -> Self {
        Self {
            account_connected: ResourceState::Loading,
            account_connected_fetcher: None,
        }
    }

    pub fn size(&self) -> u16 {
        AccountConnectedWidget {
            account: self.account_connected.get(),
        }
        .size()
    }
}

impl Component for AccountConnectedComponent {
    fn init(&mut self, ctx: &ComponentContext) {
        self.account_connected_fetcher = {
            let client = ctx.client.clone();
            Some(Fetcher::new(async move { client.get_user().await }))
        };
    }

    fn update(&mut self) {
        if let ResourceState::Loading = self.account_connected
            && let Some(account_connected_fetcher) = self.account_connected_fetcher.as_mut()
            && let Some(account) = account_connected_fetcher.try_get()
        {
            self.account_connected = ResourceState::Loaded(account);
        }
    }

    fn handle_event_key(&mut self, key_event: KeyEvent) {}

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = AccountConnectedWidget {
            account: self.account_connected.get(),
        };
        frame.render_widget(widget, area);
    }
}

pub struct AccountConnectedWidget<'a> {
    pub account: Option<&'a Account>,
}

impl AccountConnectedWidget<'_> {
    fn formatted_text(&self) -> String {
        let name = self
            .account
            .as_ref()
            .and_then(|account| account.display_name.as_deref())
            .unwrap_or(LOADING_TEXT);

        format!("  {} ", name)
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
