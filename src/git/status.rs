use crate::error::{GitFlowError, Result};
use git2::{Repository, Status, StatusOptions};

/// StatusEntry represents a single file status from the repository
#[derive(Debug)]
pub struct StatusEntry {
    pub path: String,
    pub status: Status,
}

/// Get the status of files in the repository
pub fn get_repo_status(repo: &Repository) -> Result<Vec<StatusEntry>> {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);
    status_opts.recurse_untracked_dirs(true);
    status_opts.include_unmodified(false);
    status_opts.include_ignored(false);

    let statuses = repo.statuses(Some(&mut status_opts))?;

    let mut result = Vec::new();
    for entry in statuses.iter() {
        let path = entry
            .path()
            .map(String::from)
            .ok_or_else(|| GitFlowError::Git(git2::Error::from_str("Invalid path")))?;

        result.push(StatusEntry {
            path,
            status: entry.status(),
        });
    }

    Ok(result)
}
