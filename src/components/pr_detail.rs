use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
    components::{
        Component, KeyBinding, KeyEventResponse,
        approval_popup::ApprovalPopupComponent,
    },
    fetcher::{Fetcher, ResourceState},
    models::{
        CommitStatusState, PaginatedCommitStatuses, ParticipantState, PullRequest, PullRequestState,
    },
};

pub struct PrDetailComponent {
    pr: PullRequest,
    pr_detail: ResourceState<PullRequest>,
    pr_detail_fetcher: Option<Fetcher<PullRequest>>,
    statuses: ResourceState<PaginatedCommitStatuses>,
    statuses_fetcher: Option<Fetcher<PaginatedCommitStatuses>>,
    scroll_offset: u16,
    bitbucket_client: Arc<BitbucketClient>,
    bitbucket_repo: Arc<BitbucketRepo>,
    user_uuid: String,
    approval_popup: Option<ApprovalPopupComponent>,
}

impl PrDetailComponent {
    pub fn new(
        pr: PullRequest,
        bitbucket_client: Arc<BitbucketClient>,
        bitbucket_repo: Arc<BitbucketRepo>,
        user_uuid: String,
    ) -> Self {
        Self {
            pr,
            pr_detail: ResourceState::Loading,
            pr_detail_fetcher: None,
            statuses: ResourceState::Loading,
            statuses_fetcher: None,
            scroll_offset: 0,
            bitbucket_client,
            bitbucket_repo,
            user_uuid,
            approval_popup: None,
        }
    }

    fn current_user_state(&self) -> Option<&ParticipantState> {
        let pr = self.pr_detail.get().unwrap_or(&self.pr);
        pr.participants
            .iter()
            .find(|p| {
                p.user
                    .as_ref()
                    .and_then(|u| u.uuid.as_ref())
                    .map_or(false, |uuid| uuid == &self.user_uuid)
            })
            .and_then(|p| p.state.as_ref())
    }

    fn open_approval_popup(&mut self) {
        let current_state = self.current_user_state().cloned();
        let mut popup = ApprovalPopupComponent::new(
            current_state.as_ref(),
            self.bitbucket_client.clone(),
            self.bitbucket_repo.clone(),
            self.pr.id,
        );
        popup.init();
        self.approval_popup = Some(popup);
    }

    fn fetch_data(&mut self) {
        let pr_id = self.pr.id;

        let client = self.bitbucket_client.clone();
        let repo = self.bitbucket_repo.clone();
        self.pr_detail = ResourceState::Loading;
        self.pr_detail_fetcher = Some(Fetcher::new(async move {
            client.get_pull_request(&repo, pr_id).await
        }));

        let client = self.bitbucket_client.clone();
        let repo = self.bitbucket_repo.clone();
        self.statuses = ResourceState::Loading;
        self.statuses_fetcher = Some(Fetcher::new(async move {
            client.get_pull_request_statuses(&repo, pr_id).await
        }));
    }
}

impl Component for PrDetailComponent {
    fn init(&mut self) {
        self.fetch_data();
    }

    fn update(&mut self) {
        if let Some(popup) = self.approval_popup.as_mut() {
            popup.update();
            if popup.is_done() {
                self.approval_popup = None;
                self.fetch_data();
            }
            return;
        }
        if let ResourceState::Loading = self.pr_detail
            && let Some(fetcher) = self.pr_detail_fetcher.as_mut()
            && let Some(pr) = fetcher.try_get()
        {
            self.pr_detail = ResourceState::Loaded(pr);
        }
        if let ResourceState::Loading = self.statuses
            && let Some(fetcher) = self.statuses_fetcher.as_mut()
            && let Some(statuses) = fetcher.try_get()
        {
            self.statuses = ResourceState::Loaded(statuses);
        }
    }

    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse {
        if let Some(popup) = self.approval_popup.as_mut() {
            return match popup.handle_event_key(key_event) {
                KeyEventResponse::Consumed => KeyEventResponse::Consumed,
                KeyEventResponse::Ignored => {
                    self.approval_popup = None;
                    KeyEventResponse::Consumed
                }
            };
        }

        match key_event.code {
            KeyCode::Char('a') => {
                self.open_approval_popup();
                KeyEventResponse::Consumed
            }
            KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                KeyEventResponse::Consumed
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                KeyEventResponse::Consumed
            }
            KeyCode::Esc | KeyCode::Char('q') => KeyEventResponse::Ignored,
            _ => KeyEventResponse::Consumed,
        }
    }

    fn keybindings(&self) -> Vec<KeyBinding> {
        vec![
            KeyBinding {
                key: "Esc/q",
                description: "Back to list",
            },
            KeyBinding {
                key: "Up/Down",
                description: "Scroll",
            },
            KeyBinding {
                key: "a",
                description: "Change approval",
            },
        ]
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = PrDetailWidget {
            pr: &self.pr,
            pr_detail: self.pr_detail.get(),
            statuses: self.statuses.get(),
            scroll_offset: self.scroll_offset,
        };
        frame.render_widget(widget, area);

        if let Some(popup) = &self.approval_popup {
            popup.render(frame, area);
        }
    }
}

struct PrDetailWidget<'a> {
    pr: &'a PullRequest,
    pr_detail: Option<&'a PullRequest>,
    statuses: Option<&'a PaginatedCommitStatuses>,
    scroll_offset: u16,
}

impl Widget for PrDetailWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        let state_str = self
            .pr
            .state
            .as_ref()
            .map_or("?".into(), |s| s.to_string().to_ascii_uppercase());
        let state_style = match self.pr.state.as_ref() {
            Some(PullRequestState::Open) => Style::new().fg(Color::Blue),
            Some(PullRequestState::Merged) => Style::new().fg(Color::Green),
            Some(PullRequestState::Declined) => Style::new().fg(Color::Red),
            _ => Style::default(),
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" #{}", self.pr.id),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(state_str, state_style.add_modifier(Modifier::BOLD)),
        ]));

        if let Some(title) = &self.pr.title {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(title.clone(), Style::default().add_modifier(Modifier::BOLD)),
            ]));
        }

        let source = self
            .pr
            .source
            .as_ref()
            .and_then(|e| e.branch.as_ref())
            .map_or("?", |b| b.name.as_str());
        let dest = self
            .pr
            .destination
            .as_ref()
            .and_then(|e| e.branch.as_ref())
            .map_or("?", |b| b.name.as_str());
        lines.push(Line::styled(
            format!(" {} -> {}", source, dest),
            Style::default().add_modifier(Modifier::DIM),
        ));

        let author = self
            .pr
            .author
            .as_ref()
            .and_then(|a| a.display_name.as_ref())
            .map_or("?", |n| n.as_str());

        let detail_pr = self.pr_detail.unwrap_or(self.pr);
        let created = detail_pr
            .created_on
            .as_deref()
            .and_then(|s| s.get(..10))
            .unwrap_or("...");
        let updated = detail_pr
            .updated_on
            .as_deref()
            .and_then(|s| s.get(..10))
            .unwrap_or("...");

        lines.push(Line::from(vec![
            Span::raw(format!(" {}", author)),
            Span::styled(
                format!("  |  Created: {}  |  Updated: {}", created, updated),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ]));

        lines.push(Line::raw(""));

        if let Some(detail) = self.pr_detail {
            let actionable: Vec<_> = detail
                .participants
                .iter()
                .filter(|p| p.state.is_some())
                .collect();

            if !actionable.is_empty() {
                lines.push(Line::styled(
                    " Reviewers",
                    Style::default().add_modifier(Modifier::UNDERLINED),
                ));
                for p in &actionable {
                    let (icon, style) = match &p.state {
                        Some(ParticipantState::Approved) => ("v", Style::new().fg(Color::Green)),
                        Some(ParticipantState::ChangesRequested) => {
                            ("!", Style::new().fg(Color::Red))
                        }
                        None => unreachable!(),
                    };
                    let name = p
                        .user
                        .as_ref()
                        .and_then(|u| u.display_name.as_ref())
                        .map_or("?", |n| n.as_str());
                    let state_label = match &p.state {
                        Some(ParticipantState::Approved) => "approved",
                        Some(ParticipantState::ChangesRequested) => "changes requested",
                        None => unreachable!(),
                    };
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled(format!("[{}]", icon), style),
                        Span::raw(format!(" {} ({})", name, state_label)),
                    ]));
                }
                lines.push(Line::raw(""));
            }
        } else {
            lines.push(Line::styled(
                " Reviewers",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ));
            lines.push(Line::styled(
                "   ...",
                Style::default().add_modifier(Modifier::DIM),
            ));
            lines.push(Line::raw(""));
        }

        if let Some(statuses) = self.statuses {
            if !statuses.values.is_empty() {
                lines.push(Line::styled(
                    " Build Status",
                    Style::default().add_modifier(Modifier::UNDERLINED),
                ));
                for s in &statuses.values {
                    let (icon, style) = match s.state {
                        CommitStatusState::Successful => ("v", Style::new().fg(Color::Green)),
                        CommitStatusState::Failed => ("x", Style::new().fg(Color::Red)),
                        CommitStatusState::InProgress => ("~", Style::new().fg(Color::Yellow)),
                        CommitStatusState::Stopped => ("-", Style::new().fg(Color::Gray)),
                    };
                    let label = s.name.as_deref().unwrap_or(&s.key);
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled(format!("[{}]", icon), style),
                        Span::raw(format!(" {}", label)),
                    ]));
                }
                lines.push(Line::raw(""));
            }
        } else {
            lines.push(Line::styled(
                " Build Status",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ));
            lines.push(Line::styled(
                "   ...",
                Style::default().add_modifier(Modifier::DIM),
            ));
            lines.push(Line::raw(""));
        }

        let summary_raw = self.pr.summary.as_ref().and_then(|s| s.raw.as_deref());
        if let Some(raw) = summary_raw {
            lines.push(Line::styled(
                " Description",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ));
            for line in raw.lines() {
                lines.push(Line::raw(format!("   {}", line)));
            }
        }

        Paragraph::new(lines)
            .scroll((self.scroll_offset, 0))
            .render(area, buf);
    }
}
