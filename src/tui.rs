use std::time::Duration;

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
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    bitbucket_api::Account, bitbucket_client::BitbucketClient, bitbucket_repo::BitbucketRepo,
};

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: BitbucketRepo,
    current_user: Option<Account>,
    bitbucket_client: BitbucketClient,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub fn new(repo_path: Option<String>) -> anyhow::Result<Self> {
        let repo_path = repo_path.unwrap_or(String::from("."));
        let bitbucket_repo = BitbucketRepo::new(&repo_path)?;
        Ok(Self {
            is_running: false,
            event_stream: EventStream::default(),
            selected_tab: SelectedTab::default(),
            repo_path,
            bitbucket_repo,
            current_user: None,
            bitbucket_client: BitbucketClient::from_env()?,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);

        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let bitbucket_client = self.bitbucket_client.clone();
        tokio::spawn(async move {
            let user = bitbucket_client.get_user().await;
            tx.send(user).ok()
        });
        self.is_running = true;
        while self.is_running {
            if let Ok(Ok(user)) = rx.try_recv() {
                self.current_user = Some(user);
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

        let current_tab_name = self.selected_tab.to_string();
        frame.render_widget(
            Paragraph::new(format!("in tab: {}", current_tab_name)),
            main_area,
        );

        let [app_title, user_name, repo_slug] =
            Layout::horizontal([Max(9), Max(9), Fill(1)]).areas(footer);

        frame.render_widget(
            Paragraph::new(" bb-dash ").style(Style::default().reversed().fg(Color::Blue).bold()),
            app_title,
        );

        let user_name_content = if let Some(user) = &self.current_user {
            user.display_name.clone()
        } else {
            String::from("...")
        };

        frame.render_widget(
            Paragraph::new(format!(" {} ", user_name_content)).style(Style::default().reversed()),
            user_name,
        );

        frame.render_widget(Paragraph::new(self.bitbucket_repo.slug()), repo_slug);
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
