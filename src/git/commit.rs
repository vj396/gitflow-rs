use crate::error::{GitFlowError, Result};
use git2::{Repository, StatusOptions};
use log::info;
use std::path::Path;

/// Commit changes to the repository
pub fn commit_changes(repo: &Repository, message: &str) -> Result<()> {
    // Stage all files
    let mut index = repo.index()?;
    
    // Add all files (including new, modified, and deleted)
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);
    status_opts.recurse_untracked_dirs(true);
        
    let statuses = repo.statuses(Some(&mut status_opts))?;
    for entry in statuses.iter() {
        let path = match entry.path() {
            Some(p) => p,
            None => continue,
        };
        
        if entry.status().is_wt_new() || 
           entry.status().is_wt_modified() || 
           entry.status().is_wt_renamed() || 
           entry.status().is_wt_typechange() {
            index.add_path(Path::new(path))?;
        } else if entry.status().is_wt_deleted() {
            index.remove_path(Path::new(path))?;
        }
    }
    
    // Write the index to disk
    let oid = index.write_tree()?;
    
    // Create the commit
    let signature = repo.signature()?;
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };
    
    let tree = repo.find_tree(oid)?;
    
    if let Some(parent) = parent_commit {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?;
    } else {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[],
        )?;
    }
    
    // Make sure HEAD points to the new commit
    index.write()?;
    
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
