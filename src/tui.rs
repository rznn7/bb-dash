use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

pub enum CurrentScreen {
    MyPullRequests,
    NeedMyReview,
}

pub struct App {
    is_running: bool,
    event_stream: EventStream,

    current_tab: CurrentScreen,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    fn switch_current_tab(&mut self) {
        self.current_tab = match self.current_tab {
            CurrentScreen::MyPullRequests => CurrentScreen::NeedMyReview,
            CurrentScreen::NeedMyReview => CurrentScreen::MyPullRequests,
        }
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
        frame.render_widget(Paragraph::new("Hello bb-dash!").centered(), frame.area())
    }

    async fn handle_crossterm_events(&mut self) -> Result<(), anyhow::Error> {
        let event = self.event_stream.next().fuse().await;
        if let Some(Ok(Event::Key(key_event))) = event {
            self.handle_key_event(key_event)
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('q') | KeyCode::Char('Q')) => self.quit(),
            (_, KeyCode::Esc) => self.quit(),
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.is_running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            is_running: Default::default(),
            event_stream: Default::default(),
            current_tab: CurrentScreen::MyPullRequests,
        }
    }
}
