use colored::Colorize;
use git2::Repository;

pub struct GitInfo {
    repo: Repository,
}

#[derive(Default)]
struct RepoStatus {
    modified: usize,
    staged: usize,
    untracked: usize,
}

impl GitInfo {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    fn get_status(&self) -> RepoStatus {
        let mut status = RepoStatus::default();

        if let Ok(statuses) = self.repo.statuses(None) {
            for entry in statuses.iter() {
                match entry.status() {
                    s if s.is_wt_modified() => status.modified += 1,
                    s if s.is_index_modified() => status.staged += 1,
                    s if s.is_wt_new() => status.untracked += 1,
                    _ => {}
                }
            }
        }

        status
    }

    pub fn get_info(&self) -> String {
        let branch = self
            .repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| String::from("HEAD"));

        let mut status_parts = Vec::new();
        let status = self.get_status();

        if status.modified > 0 {
            status_parts.push(format!(" !{}", status.modified));
        }
        if status.staged > 0 {
            status_parts.push(format!(" +{}", status.staged));
        }
        if status.untracked > 0 {
            status_parts.push(format!(" ?{}", status.untracked));
        }

        let status_str = if !status_parts.is_empty() {
            status_parts.join("").red().to_string()
        } else {
            String::new()
        };

        format!("{}{}", branch.green(), status_str)
    }
}
