use anyhow::{Context, Result, bail};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Clone)]
pub struct BitbucketRepo {
    workspace: String,
    slug: String,
    path: PathBuf,
}

impl BitbucketRepo {
    pub fn new(repo_path: &str) -> Result<Self> {
        Self::verify_git_repo(repo_path)?;
        let (workspace, slug) = Self::get_repo_workspace_and_slug(repo_path)?;
        Ok(Self {
            workspace,
            slug,
            path: PathBuf::from(repo_path),
        })
    }

    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn current_branch(&self) -> Result<String> {
        let output = process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.path)
            .output()
            .context("failed to run git rev-parse")?;
        if !output.status.success() {
            bail!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn checkout_branch(&self, branch: &str) -> Result<()> {
        let fetch = process::Command::new("git")
            .args(["fetch", "origin", branch])
            .current_dir(&self.path)
            .output()
            .context("failed to run git fetch")?;
        if !fetch.status.success() {
            bail!("{}", first_err_line(&fetch.stderr, "git fetch failed"));
        }
        let checkout = process::Command::new("git")
            .args(["checkout", branch])
            .current_dir(&self.path)
            .output()
            .context("failed to run git checkout")?;
        if !checkout.status.success() {
            bail!(
                "{}",
                first_err_line(&checkout.stderr, "git checkout failed")
            );
        }
        Ok(())
    }

    fn verify_git_repo(repo_path: &str) -> Result<()> {
        let output = process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(repo_path)
            .output()
            .context("failed to execute git command")?;

        if !output.status.success() {
            bail!("not in a git repository");
        }

        Ok(())
    }

    fn get_repo_workspace_and_slug(repo_path: &str) -> Result<(String, String)> {
        let output = process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(repo_path)
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

fn first_err_line(stderr: &[u8], fallback: &str) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(fallback)
        .trim()
        .to_string()
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
