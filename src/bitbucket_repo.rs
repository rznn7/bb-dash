use anyhow::{Context, Result, bail};
use regex::Regex;
use std::process;

pub struct BitbucketRepo {
    workspace: String,
    slug: String,
}

impl BitbucketRepo {
    pub fn new() -> Result<Self> {
        Self::verify_git_repo()?;
        let (workspace, slug) = Self::get_repo_workspace_and_slug()?;
        Ok(Self { workspace, slug })
    }

    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    fn verify_git_repo() -> Result<()> {
        let output = process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .context("failed to execute git command")?;

        if !output.status.success() {
            bail!("not in a git repository");
        }

        Ok(())
    }

    fn get_repo_workspace_and_slug() -> Result<(String, String)> {
        let output = process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .context("failed to execute git command")?;
        let remote_url = String::from_utf8(output.stdout)
            .context("failed to convert git origin url to String")?
            .trim()
            .to_string();

        Self::parse_repo_url(&remote_url)
    }

    fn parse_repo_url(url: &str) -> Result<(String, String)> {
        let re = Regex::new(r"[:/]([^/]+)/([^/\s]+?)(\.git)?$").unwrap();
        let Some(captures) = re.captures(url) else {
            bail!("failed to match repository workspace and slug")
        };
        let workspace = captures[1].to_string();
        let repo_slug = captures[2].to_string();
        Ok((workspace, repo_slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@bitbucket.org:myworkspace/my-repo.git";
        let (workspace, slug) = BitbucketRepo::parse_repo_url(url).unwrap();
        assert_eq!(workspace, "myworkspace");
        assert_eq!(slug, "my-repo");
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://bitbucket.org/myworkspace/my-repo.git";
        let (workspace, slug) = BitbucketRepo::parse_repo_url(url).unwrap();
        assert_eq!(workspace, "myworkspace");
        assert_eq!(slug, "my-repo");
    }

    #[test]
    fn test_parse_without_git_suffix() {
        let url = "git@bitbucket.org:myworkspace/my-repo";
        let (workspace, slug) = BitbucketRepo::parse_repo_url(url).unwrap();
        assert_eq!(workspace, "myworkspace");
        assert_eq!(slug, "my-repo");
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        assert!(BitbucketRepo::parse_repo_url(url).is_err());
    }
}
