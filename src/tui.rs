use crate::{
    bitbucket_api::PaginatedPullRequests, bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo, models::Account,
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
    widgets::{Paragraph, Row, Table, Tabs, Widget},
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    current_account: ResourceState<Account>,
    my_pull_requests: ResourceState<PaginatedPullRequests>,
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
            current_account: ResourceState::Loading,
            my_pull_requests: ResourceState::Loading,
            bitbucket_client: BitbucketClient::from_env()?,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        let mut interval = get_app_interval();

        let mut account_connected_fetcher = {
            let client = self.bitbucket_client.clone();
            Fetcher::new(async move { client.get_user().await })
        };

        let mut my_pull_requests_fetcher = {
            let client = self.bitbucket_client.clone();
            let repo = self.bitbucket_repo.clone();
            Fetcher::new(async move { client.list_pull_requests(&repo).await })
        };

        self.is_running = true;
        while self.is_running {
            if let ResourceState::Loading = self.current_account
                && let Some(account) = account_connected_fetcher.try_get()
            {
                self.current_account = ResourceState::Loaded(account);
            }

            if let ResourceState::Loading = self.my_pull_requests
                && let Some(pull_requests) = my_pull_requests_fetcher.try_get()
            {
                self.my_pull_requests = ResourceState::Loaded(pull_requests);
            }

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
                        pull_requests: self.my_pull_requests.get(),
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
            current_account: self.current_account.get(),
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

const FRAMES_PER_SECOND: f32 = 20.0;

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

struct Fetcher<T> {
    rx: tokio::sync::oneshot::Receiver<Result<T, anyhow::Error>>,
}

impl<T> Fetcher<T> {
    fn new<F>(task: F) -> Self
    where
        F: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let task_result = task.await;
            tx.send(task_result).ok()
        });
        Self { rx }
    }

    fn try_get(&mut self) -> Option<T> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }
}

enum ResourceState<T> {
    Loading,
    Loaded(T),
    Failed,
}

impl<T> ResourceState<T> {
    fn get(&self) -> Option<&T> {
        match self {
            ResourceState::Loaded(data) => Some(data),
            _ => None,
        }
    }
}

const LOADING_TEXT: &str = "...";

struct MyPullRequestsTabWidget<'a> {
    pull_requests: Option<&'a PaginatedPullRequests>,
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
    current_account: Option<&'a Account>,
}

impl AccountConnectedWidget<'_> {
    fn formatted_text(&self) -> String {
        let name = self
            .current_account
            .as_ref()
            .and_then(|account| account.display_name.as_deref())
            .unwrap_or(LOADING_TEXT);

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
