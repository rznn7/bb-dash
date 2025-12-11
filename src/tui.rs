use crate::{
    bitbucket_api::{AppAccount, AppPaginatedPullRequests},
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{self, Fill, Length, Max, Min},
        Layout, Rect,
    },
    style::{Color, Style, Stylize},
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs, Widget},
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    current_account: Option<AppAccount>,
    my_pull_requests: Option<AppPaginatedPullRequests>,
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
            my_pull_requests: None,
            bitbucket_client: BitbucketClient::from_env()?,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        let mut interval = get_app_interval();
        let mut account_connected_fetcher = AccountConnectedFetcher::new(&self.bitbucket_client);
        let mut my_pull_requests_fetcher =
            MyPullRequestsFetcher::new(&self.bitbucket_client, &self.bitbucket_repo);

        self.is_running = true;
        while self.is_running {
            self.current_account = self
                .current_account
                .or_else(|| account_connected_fetcher.try_get());

            self.my_pull_requests = self
                .my_pull_requests
                .or_else(|| my_pull_requests_fetcher.try_get());

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

        match self.selected_tab {
            SelectedTab::MyPullRequests => {
                frame.render_widget(
                    MyPullRequestsTabWidget {
                        pull_requests: &self.my_pull_requests,
                    },
                    main_area,
                );
            }
            SelectedTab::NeedMyReview => {
                frame.render_widget(Paragraph::new("NeedMyReview tab - wip"), main_area);
            }
        }

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
    rx: tokio::sync::oneshot::Receiver<Result<AppAccount, anyhow::Error>>,
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

    fn try_get(&mut self) -> Option<AppAccount> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }
}

struct MyPullRequestsFetcher {
    rx: tokio::sync::oneshot::Receiver<Result<AppPaginatedPullRequests, anyhow::Error>>,
}

impl MyPullRequestsFetcher {
    fn new(bitbucket_client: &BitbucketClient, bitbucket_repo: &BitbucketRepo) -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let bitbucket_client = bitbucket_client.clone();
        let bitbucket_repo = bitbucket_repo.clone();
        tokio::spawn(async move {
            let user = bitbucket_client.list_pull_requests(&bitbucket_repo).await;
            tx.send(user).ok()
        });

        Self { rx }
    }

    fn try_get(&mut self) -> Option<AppPaginatedPullRequests> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }
}

const LOADING_TEXT: &str = "...";

struct MyPullRequestsTabWidget<'a> {
    pull_requests: &'a Option<AppPaginatedPullRequests>,
}

impl Widget for MyPullRequestsTabWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(pull_requests) = self.pull_requests {
            let mut max_length_id = 0;
            let mut max_length_state = 0;
            let mut max_length_title = 0;
            let mut max_length_source_branch = 0;
            let mut max_length_destination_branch = 0;
            let mut rows = Vec::new();

            for pr in &pull_requests.values {
                let id = pr.id.to_string();
                let state = pr.state.to_string();
                let title = pr.title.clone();
                let source_branch = pr.source.branch.name.clone();
                let destination_branch = pr.destination.branch.name.clone();

                max_length_id = max_length_id.max(id.chars().count());
                max_length_state = max_length_state.max(state.chars().count());
                max_length_title = max_length_title.max(title.chars().count());
                max_length_source_branch =
                    max_length_source_branch.max(source_branch.chars().count());
                max_length_destination_branch =
                    max_length_destination_branch.max(destination_branch.chars().count());

                rows.push(Row::new(vec![
                    id,
                    state,
                    title,
                    source_branch,
                    destination_branch,
                ]));
            }

            let widths = [
                Constraint::Length(max_length_id as u16),
                Constraint::Length(max_length_state as u16),
                Constraint::Length(max_length_title as u16),
                Constraint::Length(max_length_source_branch as u16),
                Constraint::Length(max_length_destination_branch as u16),
            ];
            Table::new(rows, widths).column_spacing(1).render(area, buf);
        } else {
            Paragraph::new(LOADING_TEXT).render(area, buf);
        }
    }
}

struct AccountConnectedWidget<'a> {
    current_account: &'a Option<AppAccount>,
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
