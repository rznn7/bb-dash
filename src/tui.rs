use crate::components::Component;
use crate::components::account_connected::AccountConnectedComponent;
use crate::components::current_repo::CurrentRepoComponent;
use crate::components::my_pull_requests_tab::MyPullRequestsTabComponent;
use crate::{
    bitbucket_client::BitbucketClient, bitbucket_repo::BitbucketRepo, components::ComponentContext,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{
        Constraint::{Fill, Length, Max, Min},
        Layout,
    },
    style::{Color, Style},
    widgets::{Paragraph, Tabs},
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tracing::info;

pub struct App {
    is_running: bool,
    accent_color: Color,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    bitbucket_client: BitbucketClient,
    account_component: AccountConnectedComponent,
    current_repo_component: CurrentRepoComponent,
    my_pull_requests_component: MyPullRequestsTabComponent,
}

impl App {
    pub fn new(repo_path: String, accent_color: Color) -> anyhow::Result<Self> {
        let bitbucket_repo = BitbucketRepo::new(&repo_path)?;
        Ok(Self {
            is_running: false,
            accent_color,
            event_stream: EventStream::default(),
            selected_tab: SelectedTab::default(),
            repo_path,
            bitbucket_repo,
            bitbucket_client: BitbucketClient::from_env()?,
            account_component: AccountConnectedComponent::new(),
            current_repo_component: CurrentRepoComponent::new(),
            my_pull_requests_component: MyPullRequestsTabComponent::new(),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        info!("start running");
        let mut interval = get_app_interval();
        let component_ctx = ComponentContext {
            client: self.bitbucket_client.clone(),
            repo: self.bitbucket_repo.clone(),
        };
        self.account_component.init(&component_ctx);
        self.current_repo_component.init(&component_ctx);
        self.my_pull_requests_component.init(&component_ctx);

        self.is_running = true;
        while self.is_running {
            self.account_component.update();
            self.my_pull_requests_component.update();

            tokio::select! {
                _ = interval.tick() => {terminal.draw(|frame| self.draw(frame))?;},
                next_event = self.event_stream.next() => {
                    if let Some(Ok(Event::Key(key_event))) = next_event {
                        self.handle_key_event(key_event)
                    }
                },

            }
        }
        info!("stop running");
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
                self.my_pull_requests_component.render(frame, main_area);
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
            Paragraph::new(app_title_text).style(Style::default().reversed().fg(self.accent_color)),
            app_title,
        );
        self.account_component.render(frame, user_name);
        self.current_repo_component.render(frame, repo_slug);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
            _ => match self.selected_tab {
                SelectedTab::MyPullRequests => {
                    self.my_pull_requests_component.handle_event_key(key_event)
                }
                SelectedTab::NeedMyReview => (),
            },
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
