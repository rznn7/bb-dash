use std::sync::Arc;

use anyhow::{Context, bail};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, Widget},
};
use tracing::error;

use crate::{
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
    components::{Component, KeyBinding, KeyEventResponse, pr_detail::PrDetailComponent},
    debug_pretty,
    fetcher::{Fetcher, ResourceState},
    models::{PaginatedPullRequests, ParticipantState, PullRequest, PullRequestState},
};

const LOADING_TEXT: &str = "...";

pub struct MyPullRequestsTabComponent {
    my_pull_requests: ResourceState<PaginatedPullRequests>,
    my_pull_requests_fetcher: Option<Fetcher<PaginatedPullRequests>>,
    pr_details_fetchers: Vec<(usize, Fetcher<PullRequest>)>,
    bitbucket_client: Arc<BitbucketClient>,
    bitbucket_repo: Arc<BitbucketRepo>,
    user_uuid: String,
    selected_pr_idx: usize,
    clipboard: Clipboard,
    pr_detail: Option<PrDetailComponent>,
}

impl MyPullRequestsTabComponent {
    pub fn new(
        bitbucket_client: Arc<BitbucketClient>,
        bitbucket_repo: Arc<BitbucketRepo>,
        user_uuid: String,
    ) -> Self {
        Self {
            my_pull_requests: ResourceState::Loading,
            my_pull_requests_fetcher: None,
            pr_details_fetchers: Vec::new(),
            bitbucket_client,
            bitbucket_repo,
            user_uuid,
            selected_pr_idx: 0,
            clipboard: Clipboard::new().expect("failed to create Clipboard instance"),
            pr_detail: None,
        }
    }

    fn fetch_pull_requests(&mut self) {
        self.my_pull_requests = ResourceState::Loading;
        self.pr_details_fetchers.clear();
        self.my_pull_requests_fetcher = {
            let client = self.bitbucket_client.clone();
            let repo = self.bitbucket_repo.clone();
            // (no filtering for now)
            // let q = format!("author.uuid = \"{}\" AND state = \"open\"", self.user_uuid);
            let q = String::new();
            Some(Fetcher::new(async move {
                client
                    .list_pull_requests(&repo, None, Some(&q), Some(10))
                    .await
            }))
        };
    }

    fn spawn_pr_detail_fetchers(&mut self) {
        let Some(paginated) = self.my_pull_requests.get() else {
            return;
        };
        let Some(prs) = paginated.values.as_ref() else {
            return;
        };
        self.pr_details_fetchers = prs
            .iter()
            .enumerate()
            .map(|(i, pr)| {
                let client = self.bitbucket_client.clone();
                let repo = self.bitbucket_repo.clone();
                let pr_id = pr.id;
                (i, Fetcher::new(async move {
                    client.get_pull_request(&repo, pr_id).await
                }))
            })
            .collect();
    }

    fn select_pr_down(&mut self) {
        if let Some(paginated_prs) = self.my_pull_requests.get()
            && let Some(prs) = paginated_prs.values.as_ref()
            && self.selected_pr_idx < prs.len() - 1
        {
            self.selected_pr_idx += 1;
        }
    }

    fn select_pr_up(&mut self) {
        if self.selected_pr_idx > 0 {
            self.selected_pr_idx -= 1;
        }
    }

    fn copy_pr_link_to_clipboard(&mut self) {
        let href = match self.get_selected_pr() {
            Ok(pr) => pr
                .links
                .as_ref()
                .and_then(|link| link.html.href.as_ref())
                .cloned(),
            Err(e) => {
                error!("could not get selected pr: {e}");
                return;
            }
        };

        let Some(href) = href else {
            error!("PR has no html link");
            return;
        };

        if let Err(e) = self.clipboard.set_text(href) {
            error!("failed to copy pr link to clipboard: {e}");
        }
    }

    fn open_pr_in_browser(&self) {
        if let Err(e) = self
            .get_selected_pr()
            .and_then(|pr| {
                pr.links
                    .as_ref()
                    .and_then(|link| link.html.href.as_ref())
                    .context("PR has no html link")
            })
            .and_then(|href_html| open::that(href_html).context("failed to open browser"))
        {
            error!("could not open selected pr: {e}");
        }
    }

    fn open_pr_detail(&mut self) {
        if let Ok(pr) = self.get_selected_pr() {
            let mut detail = PrDetailComponent::new(
                pr.clone(),
                self.bitbucket_client.clone(),
                self.bitbucket_repo.clone(),
                self.user_uuid.clone(),
            );
            detail.init();
            self.pr_detail = Some(detail);
        }
    }

    fn get_selected_pr(&self) -> anyhow::Result<&PullRequest> {
        if let Some(paginated_prs) = self.my_pull_requests.get()
            && let Some(prs) = paginated_prs.values.as_ref()
            && let Some(selected_pr) = prs.get(self.selected_pr_idx)
        {
            Ok(selected_pr)
        } else {
            bail!("failed to get selected_pr")
        }
    }
}

impl Component for MyPullRequestsTabComponent {
    fn init(&mut self) {
        self.fetch_pull_requests();
    }

    fn update(&mut self) {
        if let Some(detail) = self.pr_detail.as_mut() {
            detail.update();
            return;
        }
        if let ResourceState::Loading = self.my_pull_requests
            && let Some(pr_fetcher) = self.my_pull_requests_fetcher.as_mut()
            && let Some(pr) = pr_fetcher.try_get()
        {
            self.my_pull_requests = ResourceState::Loaded(pr);
            self.spawn_pr_detail_fetchers();
        }

        // Poll detail fetchers and update participants as they resolve
        if let ResourceState::Loaded(ref mut paginated) = self.my_pull_requests {
            if let Some(ref mut prs) = paginated.values {
                for (idx, fetcher) in &mut self.pr_details_fetchers {
                    if let Some(detail) = fetcher.try_get() {
                        if let Some(pr) = prs.get_mut(*idx) {
                            pr.participants = detail.participants;
                        }
                    }
                }
            }
        }
    }

    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse {
        if let Some(detail) = self.pr_detail.as_mut() {
            return match detail.handle_event_key(key_event) {
                KeyEventResponse::Consumed => KeyEventResponse::Consumed,
                KeyEventResponse::Ignored => {
                    self.pr_detail = None;
                    KeyEventResponse::Consumed
                }
            };
        }

        match key_event.code {
            KeyCode::Char('r') => {
                self.fetch_pull_requests();
                KeyEventResponse::Consumed
            }
            KeyCode::Down => {
                self.select_pr_down();
                KeyEventResponse::Consumed
            }
            KeyCode::Up => {
                self.select_pr_up();
                KeyEventResponse::Consumed
            }
            KeyCode::Char('o') => {
                self.open_pr_in_browser();
                KeyEventResponse::Consumed
            }
            KeyCode::Char('y') => {
                self.copy_pr_link_to_clipboard();
                KeyEventResponse::Consumed
            }
            KeyCode::Enter => {
                self.open_pr_detail();
                KeyEventResponse::Consumed
            }
            _ => KeyEventResponse::Ignored,
        }
    }

    fn keybindings(&self) -> Vec<KeyBinding> {
        if let Some(detail) = &self.pr_detail {
            return detail.keybindings();
        }
        vec![
            KeyBinding {
                key: "Enter",
                description: "View PR details",
            },
            KeyBinding {
                key: "r",
                description: "Refresh pull requests",
            },
            KeyBinding {
                key: "Down",
                description: "Select next PR",
            },
            KeyBinding {
                key: "Up",
                description: "Select previous PR",
            },
            KeyBinding {
                key: "o",
                description: "Open PR in browser",
            },
            KeyBinding {
                key: "y",
                description: "Copy PR link",
            },
        ]
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(detail) = &self.pr_detail {
            detail.render(frame, area);
            return;
        }
        let widget = MyPullRequestsTabWidget {
            pull_requests: self.my_pull_requests.get(),
            selected_pr_idx: Some(self.selected_pr_idx),
        };
        frame.render_widget(widget, area);
    }
}

pub struct MyPullRequestsTabWidget<'a> {
    pub pull_requests: Option<&'a PaginatedPullRequests>,
    pub selected_pr_idx: Option<usize>,
}

impl Widget for MyPullRequestsTabWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(pull_requests) = self.pull_requests {
            let prs = pull_requests.values.as_deref().unwrap_or(&[]);

            let mut max_id_len = 0usize;
            let mut max_state_len = 0usize;
            let mut max_author_len = 0usize;
            let mut max_approval_len = 0usize;

            let labels: Vec<_> = prs
                .iter()
                .map(|pr| {
                    let id_label = format!("#{}", pr.id);
                    let state_label = pr
                        .state
                        .as_ref()
                        .map_or(String::from("?"), |s| s.to_string().to_ascii_lowercase());
                    let title_label = pr.title.as_ref().map_or(String::from("?"), |t| t.clone());
                    let branch_label = pr
                        .destination
                        .as_ref()
                        .and_then(|d| d.branch.as_ref())
                        .map_or(String::from("?"), |b| b.name.clone());
                    let author_label = pr
                        .author
                        .as_ref()
                        .and_then(|account| account.display_name.as_ref())
                        .map_or(String::from("?"), |display_name| display_name.clone());

                    debug_pretty!(pr);

                    let approved_count = pr
                        .participants
                        .iter()
                        .filter(|p| matches!(p.state, Some(ParticipantState::Approved)))
                        .count();
                    let changes_count = pr
                        .participants
                        .iter()
                        .filter(|p| matches!(p.state, Some(ParticipantState::ChangesRequested)))
                        .count();

                    let approval_label = match (approved_count, changes_count) {
                        (0, 0) => "-".to_string(),
                        (a, 0) => format!("✓{a}"),
                        (0, c) => format!("!{c}"),
                        (a, c) => format!("✓{a} !{c}"),
                    };

                    max_id_len = max_id_len.max(id_label.len());
                    max_state_len = max_state_len.max(state_label.len());
                    max_author_len = max_author_len.max(author_label.len());
                    max_approval_len = max_approval_len.max(approval_label.len());

                    (
                        id_label,
                        state_label,
                        title_label,
                        branch_label,
                        author_label,
                        approval_label,
                        approved_count,
                        changes_count,
                        &pr.state,
                    )
                })
                .collect();

            let rows: Vec<_> = labels
                .iter()
                .enumerate()
                .map(
                    |(
                        i,
                        (
                            id_label,
                            state_label,
                            title_label,
                            branch_label,
                            author_label,
                            _approval_label,
                            approved_count,
                            changes_count,
                            state,
                        ),
                    )| {
                        let row_style = if self.selected_pr_idx == Some(i) {
                            Style::new().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };

                        let state_label_style = match state {
                            Some(PullRequestState::Open) => Style::new().fg(Color::Blue),
                            Some(PullRequestState::Merged) => Style::new().fg(Color::Green),
                            Some(PullRequestState::Declined) => Style::new().fg(Color::Red),
                            Some(PullRequestState::Superseded) => Style::new().fg(Color::Gray),
                            _ => Style::default(),
                        }
                        .add_modifier(Modifier::DIM);

                        let approval_cell = match (*approved_count, *changes_count) {
                            (0, 0) => Cell::from(
                                Span::from("-").style(Style::new().add_modifier(Modifier::DIM)),
                            ),
                            (a, 0) => Cell::from(Span::styled(
                                format!("✓{a}"),
                                Style::new().fg(Color::Green),
                            )),
                            (0, c) => Cell::from(Span::styled(
                                format!("!{c}"),
                                Style::new().fg(Color::Yellow),
                            )),
                            (a, c) => Cell::from(Line::from(vec![
                                Span::styled(format!("✓{a}"), Style::new().fg(Color::Green)),
                                Span::raw(" "),
                                Span::styled(format!("!{c}"), Style::new().fg(Color::Yellow)),
                            ])),
                        };

                        Row::new(vec![
                            Cell::from(
                                Span::from(id_label.as_str())
                                    .style(Style::new().add_modifier(Modifier::DIM)),
                            ),
                            Cell::from(Span::from(author_label.as_str())),
                            Cell::from(Span::from(state_label.as_str()).style(state_label_style)),
                            approval_cell,
                            Cell::from(Line::from(vec![
                                Span::from(title_label.as_str()),
                                Span::from(format!("  {branch_label}"))
                                    .style(Style::new().add_modifier(Modifier::DIM)),
                            ])),
                        ])
                        .style(row_style)
                    },
                )
                .collect();

            let widths = [
                Constraint::Length(max_id_len as u16),
                Constraint::Length(max_author_len as u16),
                Constraint::Length(max_state_len as u16),
                Constraint::Length(max_approval_len as u16),
                Constraint::Fill(1),
            ];
            Table::new(rows, widths).column_spacing(1).render(area, buf);
        } else {
            Paragraph::new(LOADING_TEXT).render(area, buf);
        }
    }
}
