use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::components::{Component, KeyBinding, KeyEventResponse};

pub struct HelpPopupComponent {
    filter_text: String,
    is_filtering: bool,
    global_keybindings: Vec<KeyBinding>,
    tab_keybindings: Vec<KeyBinding>,
    tab_name: String,
}

impl HelpPopupComponent {
    pub fn new(
        global_keybindings: Vec<KeyBinding>,
        tab_keybindings: Vec<KeyBinding>,
        tab_name: String,
    ) -> Self {
        Self {
            filter_text: String::new(),
            is_filtering: false,
            global_keybindings,
            tab_keybindings,
            tab_name,
        }
    }
}

impl Component for HelpPopupComponent {
    fn init(&mut self) {}
    fn update(&mut self) {}

    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse {
        if self.is_filtering {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                    KeyEventResponse::Consumed
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                    KeyEventResponse::Consumed
                }
                KeyCode::Esc => {
                    self.filter_text.clear();
                    self.is_filtering = false;
                    KeyEventResponse::Consumed
                }
                KeyCode::Enter => {
                    self.is_filtering = false;
                    KeyEventResponse::Consumed
                }
                _ => KeyEventResponse::Consumed,
            }
        } else {
            match key_event.code {
                KeyCode::Char('/') => {
                    self.is_filtering = true;
                    KeyEventResponse::Consumed
                }
                _ => KeyEventResponse::Ignored,
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 60, area);
        let widget = HelpPopupWidget {
            global_keybindings: &self.global_keybindings,
            tab_keybindings: &self.tab_keybindings,
            tab_name: &self.tab_name,
            filter_text: &self.filter_text,
            is_filtering: self.is_filtering,
        };
        frame.render_widget(widget, popup_area);
    }
}

struct HelpPopupWidget<'a> {
    global_keybindings: &'a [KeyBinding],
    tab_keybindings: &'a [KeyBinding],
    tab_name: &'a str,
    filter_text: &'a str,
    is_filtering: bool,
}

impl HelpPopupWidget<'_> {
    fn matches_filter(kb: &KeyBinding, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let filter_lower = filter.to_ascii_lowercase();
        kb.key.to_ascii_lowercase().contains(&filter_lower)
            || kb.description.to_ascii_lowercase().contains(&filter_lower)
    }

    fn keybinding_line(kb: &KeyBinding) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!("  {:>12}  ", kb.key),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(kb.description.to_string()),
        ])
    }
}

impl Widget for HelpPopupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Help (? to close, / to filter) ");

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<Line> = Vec::new();

        // Global section
        lines.push(Line::styled(
            " Global",
            Style::default().add_modifier(Modifier::UNDERLINED),
        ));
        for kb in self.global_keybindings {
            if Self::matches_filter(kb, self.filter_text) {
                lines.push(Self::keybinding_line(kb));
            }
        }

        lines.push(Line::raw(""));

        // Tab section
        lines.push(Line::styled(
            format!(" {}", self.tab_name),
            Style::default().add_modifier(Modifier::UNDERLINED),
        ));
        for kb in self.tab_keybindings {
            if Self::matches_filter(kb, self.filter_text) {
                lines.push(Self::keybinding_line(kb));
            }
        }

        lines.push(Line::raw(""));

        // Filter input line
        if self.is_filtering {
            lines.push(Line::from(vec![
                Span::styled(" / ", Style::default().add_modifier(Modifier::DIM)),
                Span::raw(self.filter_text.to_string()),
                Span::styled("_", Style::default().add_modifier(Modifier::RAPID_BLINK)),
            ]));
        } else if !self.filter_text.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(" / ", Style::default().add_modifier(Modifier::DIM)),
                Span::raw(self.filter_text.to_string()),
            ]));
        } else {
            lines.push(Line::styled(
                " / to filter",
                Style::default().add_modifier(Modifier::DIM),
            ));
        }

        Paragraph::new(lines).render(inner, buf);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, vertical, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);
    let [_, horizontal, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(vertical);
    horizontal
}
