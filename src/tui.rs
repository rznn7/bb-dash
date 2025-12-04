use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::{FutureExt, StreamExt};
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

use crate::bitbucket_repo::BitbucketRepo;

pub struct App {
    is_running: bool,
    event_stream: EventStream,
    selected_tab: SelectedTab,
    repo_path: String,
    bitbucket_repo: Option<BitbucketRepo>,
}

impl App {
    pub fn new(repo_path: Option<String>) -> anyhow::Result<Self> {
        let repo_path = repo_path.unwrap_or(String::from("."));
        let bitbucket_repo = Some(BitbucketRepo::new(&repo_path)?);
        Ok(Self {
            is_running: false,
            event_stream: EventStream::default(),
            selected_tab: SelectedTab::default(),
            repo_path,
            bitbucket_repo,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), anyhow::Error> {
        self.is_running = true;
        while self.is_running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
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

        let [app_title, repo_slug] = Layout::horizontal([Max(9), Fill(1)]).areas(footer);

        frame.render_widget(
            Paragraph::new(" bb-dash ").style(Style::default().reversed().fg(Color::Blue).bold()),
            app_title,
        );

        let repo_slug_content = if let Some(bitbucket_repo) = &self.bitbucket_repo {
            bitbucket_repo.slug()
        } else {
            "loading..."
        };

        frame.render_widget(Paragraph::new(repo_slug_content), repo_slug);
    }

    async fn handle_crossterm_events(&mut self) -> Result<(), anyhow::Error> {
        let event = self.event_stream.next().fuse().await;
        if let Some(Ok(Event::Key(key_event))) = event {
            self.handle_key_event(key_event)
        }
        Ok(())
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
