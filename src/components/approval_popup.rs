use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
    components::{Component, KeyEventResponse, help_popup::centered_rect},
    fetcher::{Fetcher, ResourceState},
    models::ParticipantState,
};

const LOADING_TEXT: &str = "...";

#[derive(Debug, Clone, Copy)]
pub enum ApprovalAction {
    Approve,
    Unapprove,
    RequestChanges,
    UnrequestChanges,
}

impl ApprovalAction {
    fn label(self) -> &'static str {
        match self {
            ApprovalAction::Approve => "Approve",
            ApprovalAction::Unapprove => "Unapprove",
            ApprovalAction::RequestChanges => "Request Changes",
            ApprovalAction::UnrequestChanges => "Unrequest Changes",
        }
    }

    fn available_actions(current_state: Option<&ParticipantState>) -> Vec<ApprovalAction> {
        match current_state {
            None => vec![ApprovalAction::Approve, ApprovalAction::RequestChanges],
            Some(ParticipantState::Approved) => {
                vec![ApprovalAction::Unapprove, ApprovalAction::RequestChanges]
            }
            Some(ParticipantState::ChangesRequested) => {
                vec![ApprovalAction::Approve, ApprovalAction::UnrequestChanges]
            }
        }
    }
}

pub struct ApprovalPopupComponent {
    actions: Vec<ApprovalAction>,
    selected_idx: usize,
    request_state: ResourceState<()>,
    request_fetcher: Option<Fetcher<()>>,
    done: bool,
    bitbucket_client: Arc<BitbucketClient>,
    bitbucket_repo: Arc<BitbucketRepo>,
    pr_id: i32,
}

impl ApprovalPopupComponent {
    pub fn new(
        current_state: Option<&ParticipantState>,
        bitbucket_client: Arc<BitbucketClient>,
        bitbucket_repo: Arc<BitbucketRepo>,
        pr_id: i32,
    ) -> Self {
        Self {
            actions: ApprovalAction::available_actions(current_state),
            selected_idx: 0,
            request_state: ResourceState::Loaded(()),
            request_fetcher: None,
            done: false,
            bitbucket_client,
            bitbucket_repo,
            pr_id,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    fn submit_action(&mut self) {
        let action = self.actions[self.selected_idx];
        let client = self.bitbucket_client.clone();
        let repo = self.bitbucket_repo.clone();
        let pr_id = self.pr_id;

        self.request_state = ResourceState::Loading;
        self.request_fetcher = Some(Fetcher::new(async move {
            match action {
                ApprovalAction::Approve => client.approve_pull_request(&repo, pr_id).await,
                ApprovalAction::Unapprove => client.unapprove_pull_request(&repo, pr_id).await,
                ApprovalAction::RequestChanges => {
                    client.request_changes_pull_request(&repo, pr_id).await
                }
                ApprovalAction::UnrequestChanges => {
                    client.unrequest_changes_pull_request(&repo, pr_id).await
                }
            }
        }));
    }
}

impl Component for ApprovalPopupComponent {
    fn init(&mut self) {}

    fn update(&mut self) {
        if let ResourceState::Loading = self.request_state
            && let Some(fetcher) = self.request_fetcher.as_mut()
            && let Some(()) = fetcher.try_get()
        {
            self.request_state = ResourceState::Loaded(());
            self.done = true;
        }
    }

    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse {
        if let ResourceState::Loading = self.request_state {
            return KeyEventResponse::Consumed;
        }

        match key_event.code {
            KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                KeyEventResponse::Consumed
            }
            KeyCode::Down => {
                if self.selected_idx < self.actions.len() - 1 {
                    self.selected_idx += 1;
                }
                KeyEventResponse::Consumed
            }
            KeyCode::Enter => {
                self.submit_action();
                KeyEventResponse::Consumed
            }
            KeyCode::Esc => KeyEventResponse::Ignored,
            _ => KeyEventResponse::Consumed,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(40, 30, area);

        let widget = ApprovalPopupWidget {
            actions: &self.actions,
            selected_idx: self.selected_idx,
            is_loading: matches!(self.request_state, ResourceState::Loading),
        };
        frame.render_widget(widget, popup_area);
    }
}

struct ApprovalPopupWidget<'a> {
    actions: &'a [ApprovalAction],
    selected_idx: usize,
    is_loading: bool,
}

impl Widget for ApprovalPopupWidget<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Change Approval (Esc to close) ");

        let inner = block.inner(area);
        block.render(area, buf);

        if self.is_loading {
            Paragraph::new(LOADING_TEXT).render(inner, buf);
            return;
        }

        let lines: Vec<Line> = self
            .actions
            .iter()
            .enumerate()
            .map(|(i, action)| {
                let style = if i == self.selected_idx {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(format!("  {}  ", action.label()), style))
            })
            .collect();

        Paragraph::new(lines).render(inner, buf);
    }
}
