//! Module for retrieving Git repository status.
//!
//! This module provides functions to obtain the status of files in the repository,
//! including untracked files and modifications.
//!
//! # Details
//! Enhanced documentation is provided for easier debugging and maintenance.

use crate::error::{GitFlowError, Result};
use colored::Colorize;
use git2::{Repository, Status, StatusOptions};

/// StatusEntry represents a file's status in the repository.
#[derive(Debug)]
pub struct StatusEntry {
    pub path: String,
    pub status: Status,
}

/// Get the status of files in the repository.
///
/// # Arguments
/// * `repo`              - A reference to the Git repository.
/// * `include_untracked` - Whether to include untracked files in the status.
///
/// # Returns
/// * `Result<Vec<StatusEntry>>` - A vector of file status entries, or an error if the operation fails.
///
/// # Examples
/// ```rust
/// // Example: Retrieve status including untracked files.
/// // let statuses = get_repo_status(&repo, true)?;
/// ```
pub fn get_repo_status(repo: &Repository, include_untracked: bool) -> Result<Vec<StatusEntry>> {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(include_untracked);
    status_opts.recurse_untracked_dirs(include_untracked);
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

/// Format a status code for display
pub fn format_status_entry(status: Status) -> colored::ColoredString {
    let mut result = String::new();

    if status.is_index_new() {
        result.push('A');
    } else if status.is_index_modified() {
        result.push('M');
    } else if status.is_index_deleted() {
        result.push('D');
    } else if status.is_index_renamed() {
        result.push('R');
    } else if status.is_index_typechange() {
        result.push('T');
    } else {
        result.push(' ');
    }

    if status.is_wt_new() {
        result.push('?');
    } else if status.is_wt_modified() {
        result.push('M');
    } else if status.is_wt_deleted() {
        result.push('D');
    } else if status.is_wt_renamed() {
        result.push('R');
    } else if status.is_wt_typechange() {
        result.push('T');
    } else {
        result.push(' ');
    }

    if status.is_conflicted() {
        result.push('C');
    } else {
        result.push(' ');
    }

    match result.trim() {
        "A" | "A " | "A  " => result.green(),
        "M" | "M " | " M" | "M  " | " M " => result.yellow(),
        "D" | "D " | " D" | "D  " | " D " => result.red(),
        "??" | "? " | " ?" => "??".blue(),
        "C" | " C" | "C " | "  C" => result.bright_red(),
        _ => result.normal(),
    }
}

/// Check if the repository has any uncommitted changes
pub fn has_uncommitted_changes(repo: &Repository) -> Result<bool> {
    Ok(!get_repo_status(repo, true)?.is_empty())
}

/// Check if the repository has any merge conflicts
pub fn has_conflicts(repo: &Repository) -> Result<bool> {
    let statuses = get_repo_status(repo, true)?;

    for status_entry in statuses {
        if status_entry.status.is_conflicted() {
            return Ok(true);
        }
    }

    Ok(false)
}
