use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
}

impl Worktree {
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

pub struct WorktreeManager {
    pub repo_path: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    pub fn list(&self) -> Result<Vec<Worktree>> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("list")
            .arg("--porcelain")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git worktree list")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = str::from_utf8(&output.stdout)?;
        let mut worktrees = Vec::new();
        let mut current_worktree: Option<(PathBuf, String)> = None;
        let mut current_branch = String::new();

        for line in stdout.lines() {
            if line.starts_with("worktree ") {
                if let Some(path) = line.strip_prefix("worktree ") {
                    current_worktree = Some((PathBuf::from(path), String::new()));
                }
            } else if line.starts_with("HEAD ") {
                if let Some((_, ref mut commit)) = current_worktree {
                    *commit = line.strip_prefix("HEAD ").unwrap_or("").to_string();
                }
            } else if line.starts_with("branch ") {
                current_branch = line.strip_prefix("branch ").unwrap_or("").to_string();
            } else if line.starts_with("detached") {
                current_branch = "detached".to_string();
            } else if line.is_empty() && current_worktree.is_some() {
                if let Some((path, commit)) = current_worktree.take() {
                    worktrees.push(Worktree {
                        path,
                        commit,
                        branch: current_branch.clone(),
                        is_bare: false,
                        is_detached: current_branch == "detached",
                        is_prunable: false,
                    });
                    current_branch.clear();
                }
            }
        }

        if let Some((path, commit)) = current_worktree {
            worktrees.push(Worktree {
                path,
                commit,
                branch: current_branch.clone(),
                is_bare: false,
                is_detached: current_branch == "detached",
                is_prunable: false,
            });
        }

        Ok(worktrees)
    }

    pub fn create(
        &self,
        branch_name: &str,
        parent_branch: &str,
        path: Option<&Path>,
    ) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("worktree").arg("add").current_dir(&self.repo_path);

        if let Some(p) = path {
            cmd.arg(p);
        } else {
            cmd.arg(format!("../{}", branch_name));
        }

        cmd.arg("-b").arg(branch_name).arg(parent_branch);

        let output = cmd.output().context("Failed to execute git worktree add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    pub fn create_from_branch(&self, branch_name: &str, path: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("worktree").arg("add").current_dir(&self.repo_path);

        if let Some(p) = path {
            cmd.arg(p);
        } else {
            cmd.arg(format!("../{}", branch_name));
        }

        cmd.arg(branch_name);

        let output = cmd.output().context("Failed to execute git worktree add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    pub fn remove(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg(path)
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git worktree remove")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree remove failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    pub fn prune(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("prune")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git worktree prune")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree prune failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    pub fn get_branches(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("branch")
            .arg("-a")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to execute git branch")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git branch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = str::from_utf8(&output.stdout)?;
        let branches = stdout
            .lines()
            .filter_map(|line| {
                let branch = line.trim_start_matches('*').trim();
                if !branch.starts_with("remotes/") {
                    Some(branch.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(branches)
    }

    /// Return the repository's default branch, preferring the branch advertised
    /// by origin/HEAD. The returned name matches a local branch when possible.
    pub fn get_default_branch(&self, branches: &[String]) -> Result<String> {
        if branches.is_empty() {
            return Err(anyhow::anyhow!("No local branches are available"));
        }

        let remote_head = Command::new("git")
            .args([
                "symbolic-ref",
                "--quiet",
                "--short",
                "refs/remotes/origin/HEAD",
            ])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to determine origin's default branch")?;

        let remote_default = remote_head
            .status
            .success()
            .then(|| str::from_utf8(&remote_head.stdout))
            .transpose()?
            .map(str::trim);

        let configured_default = Command::new("git")
            .args(["config", "--get", "init.defaultBranch"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to read Git's default branch setting")?;

        let configured_default = configured_default
            .status
            .success()
            .then(|| str::from_utf8(&configured_default.stdout))
            .transpose()?
            .map(str::trim);

        Ok(choose_default_branch(
            branches,
            remote_default,
            configured_default,
        ))
    }
}

fn choose_default_branch(
    branches: &[String],
    remote_default: Option<&str>,
    configured_default: Option<&str>,
) -> String {
    let candidates = [
        remote_default.map(|branch| branch.strip_prefix("origin/").unwrap_or(branch)),
        configured_default,
        Some("main"),
        Some("master"),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| branches.iter().any(|branch| branch == candidate))
        .map(str::to_string)
        .unwrap_or_else(|| branches[0].clone())
}

#[cfg(test)]
mod tests {
    use super::choose_default_branch;

    fn branches(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn remote_head_has_highest_priority() {
        let available = branches(&["develop", "main", "release"]);

        assert_eq!(
            choose_default_branch(&available, Some("origin/release"), Some("develop")),
            "release"
        );
    }

    #[test]
    fn uses_git_configuration_when_remote_head_is_unavailable() {
        let available = branches(&["develop", "main"]);

        assert_eq!(
            choose_default_branch(&available, None, Some("develop")),
            "develop"
        );
    }

    #[test]
    fn falls_back_to_conventional_or_first_branch() {
        assert_eq!(
            choose_default_branch(&branches(&["topic", "master"]), None, None),
            "master"
        );
        assert_eq!(
            choose_default_branch(&branches(&["develop", "topic"]), None, None),
            "develop"
        );
    }
}
