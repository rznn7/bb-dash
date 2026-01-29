use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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
            let mut rows = Vec::new();

            for (i, pr) in pull_requests
                .values
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .enumerate()
            {
                let id = pr.id.to_string();
                let state = pr
                    .state
                    .as_ref()
                    .map_or(String::from("?"), |state| state.to_string());
                let title = pr
                    .title
                    .as_ref()
                    .map_or(String::from("?"), |title| title.clone());
                let source_branch = pr
                    .source
                    .as_ref()
                    .and_then(|s| s.branch.as_ref())
                    .map_or(String::from("?"), |b| b.name.clone());
                let destination_branch = pr
                    .destination
                    .as_ref()
                    .and_then(|d| d.branch.as_ref())
                    .map_or(String::from("?"), |b| b.name.clone());

                let selected_style = Style::new().add_modifier(Modifier::REVERSED);
                let unselected_style = Style::default();

                let row_style = self
                    .selected_pr_idx
                    .filter(|selected_idx| *selected_idx == i)
                    .map(|_| selected_style)
                    .unwrap_or(unselected_style);

                let state_style = match pr.state {
                    Some(PullRequestState::Open) => Style::new().bg(Color::Blue),
                    Some(PullRequestState::Merged) => Style::new().bg(Color::Green),
                    Some(PullRequestState::Declined) => Style::new().bg(Color::Red),
                    Some(PullRequestState::Superseded) => Style::new().bg(Color::Gray),
                    _ => Style::default(),
                };

                let row = Row::new(vec![
                    Cell::from(Text::from(vec![
                        Line::from(vec![
                            Span::from(format!(" {state} ")).style(state_style),
                            " ".into(),
                            title.into(),
                        ]),
                        Line::from(vec!["#".into(), id.into()]),
                    ])),
                    Cell::from(Text::from(vec![Line::from(source_branch), Line::from("")])),
                    Cell::from(Text::from(vec![
                        Line::from(destination_branch),
                        Line::from(""),
                    ])),
                ])
                .style(row_style)
                .height(2)
                .bottom_margin(1);

                rows.push(row);
            }

            let widths = [
                Constraint::Percentage(70),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ];
            Table::new(rows, widths).column_spacing(1).render(area, buf);
        } else {
            Paragraph::new(LOADING_TEXT).render(area, buf);
        }
    }
}
