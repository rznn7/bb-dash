use crate::{
    bitbucket_api::Account, bitbucket_client::BitbucketClient, bitbucket_repo::BitbucketRepo,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{Fill, Length, Max, Min},
        Layout, Rect,
    },
    style::{Color, Style, Stylize},
    widgets::{Paragraph, Tabs, Widget},
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    current_account: Option<Account>,
    bitbucket_client: BitbucketClient,
}

impl App {
    pub fn new(repo_path: Option<String>) -> anyhow::Result<Self> {
        let repo_path = repo_path.unwrap_or(String::from("."));
        let bitbucket_repo = BitbucketRepo::new(&repo_path)?;
        Ok(Self {
            is_running: false,
            event_stream: EventStream::default(),
            selected_tab: SelectedTab::default(),
            repo_path,
            bitbucket_repo,
            current_account: None,
            bitbucket_client: BitbucketClient::from_env()?,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        let mut interval = get_app_interval();
        let mut account_connected_fetcher = AccountConnectedFetcher::new(&self.bitbucket_client);

        self.is_running = true;
        while self.is_running {
            self.current_account = self
                .current_account
                .or_else(|| account_connected_fetcher.try_get());

            tokio::select! {
                _ = interval.tick() => {terminal.draw(|frame| self.draw(frame))?;},
                next_event = self.event_stream.next() => {
                    if let Some(Ok(Event::Key(key_event))) = next_event {
                        self.handle_key_event(key_event)
                    }
                },

            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, main_area, footer] =
            Layout::vertical([Length(2), Min(0), Length(1)]).areas(frame.area());

        let tab_titles: Vec<String> = SelectedTab::iter()
            .map(|tab| format!(" {} ", tab))
            .collect();
        frame.render_widget(
            Tabs::new(tab_titles)
                .select(self.selected_tab as usize)
                .highlight_style(Style::default().reversed().bold()),
            header,
        );

        let current_tab_name = self.selected_tab.to_string();
        frame.render_widget(
            Paragraph::new(format!("in tab: {}", current_tab_name)),
            main_area,
        );

        let app_title_text = String::from("  bb-dash ");
        let app_title_char_count = app_title_text.chars().count() as u16;

        let account_connected_widget = AccountConnectedWidget {
            current_account: &self.current_account,
        };

        let [app_title, user_name, repo_slug] = Layout::horizontal([
            Max(app_title_char_count),
            Max(account_connected_widget.size()),
            Fill(1),
        ])
        .areas(footer);

        frame.render_widget(
            Paragraph::new(app_title_text)
                .style(Style::default().reversed().fg(Color::Blue).bold()),
            app_title,
        );

        frame.render_widget(account_connected_widget, user_name);

        frame.render_widget(
            CurrentRepoWidget {
                bitbucket_repo: &self.bitbucket_repo,
            },
            repo_slug,
        );
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn quit(&mut self) {
        self.is_running = false;
    }
}

const FRAMES_PER_SECOND: f32 = 60.0;

fn get_app_interval() -> tokio::time::Interval {
    let period = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND);
    tokio::time::interval(period)
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "MyPullRequests")]
    MyPullRequests,
    #[strum(to_string = "NeedMyReview")]
    NeedMyReview,
}

impl SelectedTab {
    fn previous(self) -> Self {
        let current_index = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

struct AccountConnectedFetcher {
    rx: tokio::sync::oneshot::Receiver<Result<Account, anyhow::Error>>,
}

impl AccountConnectedFetcher {
    fn new(bitbucket_client: &BitbucketClient) -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let bitbucket_client = bitbucket_client.clone();
        tokio::spawn(async move {
            let user = bitbucket_client.get_user().await;
            tx.send(user).ok()
        });

        Self { rx }
    }

    fn try_get(&mut self) -> Option<Account> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }
}

const LOADING_TEXT: &str = "...";

struct AccountConnectedWidget<'a> {
    current_account: &'a Option<Account>,
}

impl AccountConnectedWidget<'_> {
    fn formatted_text(&self) -> String {
        let name = self
            .current_account
            .as_ref()
            .map_or(LOADING_TEXT, |account| &account.display_name);

        format!("  {} ", name)
    }

    fn size(&self) -> u16 {
        self.formatted_text().chars().count() as u16
    }
}

impl Widget for AccountConnectedWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.formatted_text())
            .style(Style::default().reversed())
            .render(area, buf);
    }
}

struct CurrentRepoWidget<'a> {
    bitbucket_repo: &'a BitbucketRepo,
}

impl Widget for CurrentRepoWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let formatted = format!("  {}", self.bitbucket_repo.slug());
        Paragraph::new(formatted).render(area, buf);
    }
}
