use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, Widget},
};

use crate::{
    bitbucket_client::BitbucketClient,
    bitbucket_repo::BitbucketRepo,
    components::{Component, KeyEventResponse},
    fetcher::{Fetcher, ResourceState},
    models::{PaginatedPullRequests, PullRequestState},
};

const LOADING_TEXT: &str = "...";

pub struct MyPullRequestsTabComponent {
    my_pull_requests: ResourceState<PaginatedPullRequests>,
    my_pull_requests_fetcher: Option<Fetcher<PaginatedPullRequests>>,
    bitbucket_client: Arc<BitbucketClient>,
    bitbucket_repo: Arc<BitbucketRepo>,
    pub selected_pr_idx: usize,
}

impl MyPullRequestsTabComponent {
    pub fn new(bitbucket_client: Arc<BitbucketClient>, bitbucket_repo: Arc<BitbucketRepo>) -> Self {
        Self {
            my_pull_requests: ResourceState::Loading,
            my_pull_requests_fetcher: None,
            bitbucket_client,
            bitbucket_repo,
            selected_pr_idx: 0,
        }
    }

    fn fetch_pull_requests(&mut self) {
        self.my_pull_requests = ResourceState::Loading;
        self.my_pull_requests_fetcher = {
            let client = self.bitbucket_client.clone();
            let repo = self.bitbucket_repo.clone();
            let q = Some("");
            Some(Fetcher::new(async move {
                client.list_pull_requests(&repo, None, q).await
            }))
        };
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
}

impl Component for MyPullRequestsTabComponent {
    fn init(&mut self) {
        self.fetch_pull_requests();
    }

    fn update(&mut self) {
        if let ResourceState::Loading = self.my_pull_requests
            && let Some(pr_fetcher) = self.my_pull_requests_fetcher.as_mut()
            && let Some(pr) = pr_fetcher.try_get()
        {
            self.my_pull_requests = ResourceState::Loaded(pr);
        }
    }

    fn handle_event_key(&mut self, key_event: KeyEvent) -> KeyEventResponse {
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
            _ => KeyEventResponse::Ignored,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
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

                    max_id_len = max_id_len.max(id_label.len());
                    max_state_len = max_state_len.max(state_label.len());

                    (id_label, state_label, title_label, branch_label, &pr.state)
                })
                .collect();

            let rows: Vec<_> = labels
                .iter()
                .enumerate()
                .map(
                    |(i, (id_label, state_label, title_label, branch_label, state))| {
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

                        Row::new(vec![
                            Cell::from(
                                Span::from(id_label.as_str())
                                    .style(Style::new().add_modifier(Modifier::DIM)),
                            ),
                            Cell::from(Span::from(state_label.as_str()).style(state_label_style)),
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
                Constraint::Length(max_state_len as u16),
                Constraint::Fill(1),
            ];
            Table::new(rows, widths).column_spacing(1).render(area, buf);
        } else {
            Paragraph::new(LOADING_TEXT).render(area, buf);
        }
    }
}
