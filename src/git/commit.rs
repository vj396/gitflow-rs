use crate::error::{GitFlowError, Result};
use git2::Repository;
use log::info;
use std::path::Path;

/// Commit changes to the repository
pub fn commit_changes(repo: &Repository, message: &str) -> Result<()> {
    let mut index = repo.index()?;

    // Add all modified files
    for entry in repo.statuses(None)?.iter() {
        if entry.status().is_wt_new()
            || entry.status().is_wt_modified()
            || entry.status().is_wt_deleted()
        {
            if let Some(path) = entry.path() {
                index.add_path(Path::new(path))?;
            }
        }
    }

    // Write the index
    let oid = index.write_tree()?;

    // Get the signature
    let signature = repo.signature()?;

    // Get the parent commit if it exists
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    let tree = repo.find_tree(oid)?;

    if let Some(parent) = parent_commit {
        // Create the commit with a parent
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?;
    } else {
        // Create initial commit
        repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
    }

    info!("Changes committed successfully");
    Ok(())
}

/// Get the last commit message from the repository
pub fn get_last_commit_message(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    commit
        .message()
        .map(String::from)
        .ok_or_else(|| GitFlowError::Git(git2::Error::from_str("Invalid commit message")))
}

/// Get the first line of the last commit message
pub fn get_last_commit_summary(repo: &Repository) -> Result<String> {
    let message = get_last_commit_message(repo)?;

    Ok(message.lines().next().unwrap_or("").to_string())
}
