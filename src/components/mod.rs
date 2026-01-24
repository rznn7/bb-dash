use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

pub mod account_connected;
pub mod app_title;
pub mod current_repo;
pub mod my_pull_requests_tab;

pub trait Component {
    fn init(&mut self);
    fn update(&mut self);
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse;
}

pub enum KeyEventResponse {
    Consumed,
    Ignored,
}
