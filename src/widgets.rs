use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Paragraph, Row, Table, Widget},
};

use crate::{
    bitbucket_repo::BitbucketRepo,
    models::{Account, PaginatedPullRequests},
};

const LOADING_TEXT: &str = "...";

pub struct MyPullRequestsTabWidget<'a> {
    pub pull_requests: Option<&'a PaginatedPullRequests>,
}

impl Widget for MyPullRequestsTabWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(pull_requests) = self.pull_requests {
            let mut max_length_id = 0;
            let mut max_length_state = 0;
            let mut max_length_title = 0;
            let mut max_length_source_branch = 0;
            let mut max_length_destination_branch = 0;
            let mut rows = Vec::new();

            for pr in pull_requests.values.as_deref().unwrap_or(&[]) {
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

                max_length_id = max_length_id.max(id.chars().count());
                max_length_state = max_length_state.max(state.chars().count());
                max_length_title = max_length_title.max(title.chars().count());
                max_length_source_branch =
                    max_length_source_branch.max(source_branch.chars().count());
                max_length_destination_branch =
                    max_length_destination_branch.max(destination_branch.chars().count());

                rows.push(Row::new(vec![
                    id,
                    state,
                    title,
                    source_branch,
                    destination_branch,
                ]));
            }

            let widths = [
                Constraint::Length(max_length_id as u16),
                Constraint::Length(max_length_state as u16),
                Constraint::Length(max_length_title as u16),
                Constraint::Length(max_length_source_branch as u16),
                Constraint::Length(max_length_destination_branch as u16),
            ];
            Table::new(rows, widths).column_spacing(1).render(area, buf);
        } else {
            Paragraph::new(LOADING_TEXT).render(area, buf);
        }
    }
}

pub struct AccountConnectedWidget<'a> {
    pub current_account: Option<&'a Account>,
}

impl AccountConnectedWidget<'_> {
    fn formatted_text(&self) -> String {
        let name = self
            .current_account
            .as_ref()
            .and_then(|account| account.display_name.as_deref())
            .unwrap_or(LOADING_TEXT);

        format!("  {} ", name)
    }

    pub fn size(&self) -> u16 {
        self.formatted_text().chars().count() as u16
    }
}

impl Widget for AccountConnectedWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.formatted_text())
            .style(Style::default().reversed())
            .render(area, buf);
    }
}

pub struct CurrentRepoWidget<'a> {
    pub bitbucket_repo: &'a BitbucketRepo,
}

impl Widget for CurrentRepoWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let formatted = format!("  {}", self.bitbucket_repo.slug());
        Paragraph::new(formatted).render(area, buf);
    }
}
