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
            let mut status = Vec::new();

            if let Ok(statuses) = self.repo.statuses(None) {
                let mut modified = 0;
                let mut staged = 0;
                let mut untracked = 0;

                for entry in statuses.iter() {
                    match entry.status() {
                        s if s.is_wt_modified() => modified += 1,
                        s if s.is_index_modified() => staged += 1,
                        s if s.is_wt_new() => untracked += 1,
                        _ => {}
                    }
                }

                if modified > 0 {
                    status.push(format!(" !{}", modified));
                }
                if staged > 0 {
                    status.push(format!(" +{}", staged));
                }
                if untracked > 0 {
                    status.push(format!(" ?{}", untracked));
                }
            }

            let status_str = if !status.is_empty() {
                status.join("").red().to_string()
            } else {
                String::new()
            };

            format!("{}{}", branch_name.green(), status_str)
        } else {
            String::new()
        }
    }
}
