use anyhow::{Context, Result, bail};
use regex::Regex;
use std::process;

#[derive(Clone)]
pub struct BitbucketRepo {
    workspace: String,
    slug: String,
}

impl BitbucketRepo {
    pub fn new(repo_path: &str) -> Result<Self> {
        Self::verify_git_repo(repo_path)?;
        let (workspace, slug) = Self::get_repo_workspace_and_slug(repo_path)?;
        Ok(Self { workspace, slug })
    }

    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    fn verify_git_repo(repo_path: &str) -> Result<()> {
        let mut cmd = process::Command::new("git");
        cmd.args(["rev-parse", "--is-inside-work-tree"]);

        cmd.current_dir(repo_path);

        let output = cmd.output().context("failed to execute git command")?;

        if !output.status.success() {
            bail!("not in a git repository");
        }

        Ok(())
    }

    fn get_repo_workspace_and_slug(repo_path: &str) -> Result<(String, String)> {
        let mut cmd = process::Command::new("git");
        cmd.args(["remote", "get-url", "origin"]);
        cmd.current_dir(repo_path);

        let output = cmd.output().context("failed to execute git command")?;
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
