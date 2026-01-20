use crate::components::Component;
use crate::components::account_connected::AccountConnectedComponent;
use crate::{
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
    components::ComponentContext,
    fetcher::{Fetcher, ResourceState},
    models::PaginatedPullRequests,
    widgets::{CurrentRepoWidget, MyPullRequestsTabWidget},
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{
        Constraint::{Fill, Length, Max, Min},
        Layout,
    },
    style::{Color, Style, Stylize},
    widgets::{Paragraph, Tabs},
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    bitbucket_client: BitbucketClient,
    my_pull_requests: ResourceState<PaginatedPullRequests>,
    my_pull_requests_fetcher: Option<Fetcher<PaginatedPullRequests>>,
    my_pull_requests_selected: Option<usize>,
    account_component: AccountConnectedComponent,
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
            my_pull_requests: ResourceState::Loading,
            my_pull_requests_fetcher: None,
            my_pull_requests_selected: Some(0),
            bitbucket_client: BitbucketClient::from_env()?,
            account_component: AccountConnectedComponent::new(),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        let mut interval = get_app_interval();
        let component_ctx = ComponentContext {
            client: self.bitbucket_client.clone(),
            repo: self.bitbucket_repo.clone(),
        };
        self.account_component.init(&component_ctx);

        self.load_my_pull_requests();

        self.is_running = true;
        while self.is_running {
            self.update_my_pull_requests();
            self.account_component.update();

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
                        selected_pr_idx: self.my_pull_requests_selected,
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

        let [app_title, user_name, repo_slug] = Layout::horizontal([
            Max(app_title_char_count),
            Max(self.account_component.size()),
            Fill(1),
        ])
        .areas(footer);

        frame.render_widget(
            Paragraph::new(app_title_text)
                .style(Style::default().reversed().fg(Color::Blue).bold()),
            app_title,
        );

        self.account_component.render(frame, user_name);

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
            KeyCode::Char('r') => self.load_my_pull_requests(),
            KeyCode::Down => self.change_selected_my_pull_request_down(),
            KeyCode::Up => self.change_selected_my_pull_request_up(),
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

    fn update_my_pull_requests(&mut self) {
        if let ResourceState::Loading = self.my_pull_requests
            && let Some(pull_requests_fetcher) = self.my_pull_requests_fetcher.as_mut()
            && let Some(pull_requests) = pull_requests_fetcher.try_get()
        {
            self.my_pull_requests = ResourceState::Loaded(pull_requests);
            self.my_pull_requests_selected = Some(0);
        }
    }

    fn load_my_pull_requests(&mut self) {
        self.my_pull_requests = ResourceState::Loading;
        self.my_pull_requests_fetcher = {
            let client = self.bitbucket_client.clone();
            let repo = self.bitbucket_repo.clone();
            Some(Fetcher::new(async move {
                client.list_pull_requests(&repo, None).await
            }))
        };
    }

    fn change_selected_my_pull_request_down(&mut self) {
        let Some(current_idx) = self.my_pull_requests_selected else {
            return;
        };

        let Some(pull_requests) = self.my_pull_requests.get() else {
            return;
        };

        let max_idx = pull_requests
            .values
            .as_ref()
            .map_or(0, |v| v.len().saturating_sub(1));

        self.my_pull_requests_selected = Some(current_idx.saturating_add(1).min(max_idx));
    }

    fn change_selected_my_pull_request_up(&mut self) {
        let Some(current_idx) = self.my_pull_requests_selected else {
            return;
        };

        let Some(pull_requests) = self.my_pull_requests.get() else {
            return;
        };

        let max_idx = pull_requests
            .values
            .as_ref()
            .map_or(0, |v| v.len().saturating_sub(1));

        self.my_pull_requests_selected = Some(current_idx.saturating_sub(1).min(max_idx));
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
