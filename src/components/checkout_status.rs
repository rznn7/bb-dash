use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Paragraph, Widget},
};
use tracing::error;

use crate::{bitbucket_repo::BitbucketRepo, fetcher::Fetcher};

pub enum CheckoutStatus {
    Idle,
    Running(String),
    Success(String),
    Error(String),
}

pub struct CheckoutController {
    repo: Arc<BitbucketRepo>,
    status: CheckoutStatus,
    fetcher: Option<Fetcher<Result<String, String>>>,
}

impl CheckoutController {
    pub fn new(repo: Arc<BitbucketRepo>) -> Self {
        Self {
            repo,
            status: CheckoutStatus::Idle,
            fetcher: None,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, CheckoutStatus::Running(_))
    }

    pub fn has_message(&self) -> bool {
        !matches!(self.status, CheckoutStatus::Idle)
    }

    pub fn start(&mut self, branch: Option<&str>) {
        if self.is_running() {
            return;
        }
        let Some(branch) = branch.filter(|b| !b.is_empty()) else {
            self.status = CheckoutStatus::Error("no source branch on this PR".into());
            return;
        };

        match self.repo.current_branch() {
            Ok(current) if current == branch => {
                self.status = CheckoutStatus::Success(format!("already on {branch}"));
                return;
            }
            Ok(_) => {}
            Err(e) => {
                error!("current_branch failed: {e}");
            }
        }

        let branch_for_task = branch.to_string();
        let branch_for_result = branch.to_string();
        let repo = self.repo.clone();
        self.status = CheckoutStatus::Running(branch.to_string());
        self.fetcher = Some(Fetcher::new(async move {
            let res = tokio::task::spawn_blocking(move || repo.checkout_branch(&branch_for_task))
                .await
                .map_err(|e| format!("join error: {e}"))
                .and_then(|r| r.map_err(|e| e.to_string()));
            Ok(res.map(|()| branch_for_result))
        }));
    }

    pub fn poll(&mut self) {
        if !matches!(self.status, CheckoutStatus::Running(_)) {
            return;
        }
        if let Some(fetcher) = self.fetcher.as_mut()
            && let Some(result) = fetcher.try_get()
        {
            self.fetcher = None;
            self.status = match result {
                Ok(branch) => CheckoutStatus::Success(format!("checked out {branch}")),
                Err(msg) => CheckoutStatus::Error(msg),
            };
        }
    }

    pub fn clear_if_idle_message(&mut self) {
        if matches!(
            self.status,
            CheckoutStatus::Success(_) | CheckoutStatus::Error(_)
        ) {
            self.status = CheckoutStatus::Idle;
        }
    }

    pub fn render_line(&self, area: Rect, buf: &mut Buffer) {
        let (text, style) = match &self.status {
            CheckoutStatus::Idle => return,
            CheckoutStatus::Running(b) => (
                format!(" checking out {b}..."),
                Style::default().add_modifier(Modifier::DIM),
            ),
            CheckoutStatus::Success(msg) => {
                (format!(" {msg}"), Style::default().fg(Color::Green))
            }
            CheckoutStatus::Error(msg) => (format!(" {msg}"), Style::default().fg(Color::Red)),
        };
        Paragraph::new(Span::styled(text, style)).render(area, buf);
    }
}
