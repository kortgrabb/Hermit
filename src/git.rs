use colored::Colorize;
use git2::Repository;

pub struct GitInfo {
    repo: Repository,
}

impl GitInfo {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub fn get_info(&self) -> String {
        let head = self.repo.head().ok();
        let branch = head.as_ref().and_then(|h| h.shorthand());

        if let Some(branch_name) = branch {
            let mut status = String::new();
            if let Ok(statuses) = self.repo.statuses(None) {
                if !statuses.is_empty() {
                    status.push('*');
                }
            }
            format!("[{}{}]", branch_name.yellow(), status.red())
        } else {
            String::new()
        }
    }
}
