//! Module for the 'create' command.
//!
//! This module handles creating a new branch based on the current branch or
//! a specified parent branch.
//!
//! # Details
//! This file is maintained with detailed documentation to aid future maintenance.
//! Each function includes sections for arguments, returns, and examples.

use crate::error::Result;
use crate::git;
use git2::Repository;
use log::info;

/// Handle the 'create' command to create a new branch.
///
/// # Arguments
///
/// * `repo`   - A reference to the Git repository.
/// * `name`   - The name of the new branch to create.
/// * `parent` - An optional parent branch name to base the new branch upon.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok on success; otherwise returns an error.
///
/// # Examples
///
/// ```rust
/// // Example usage:
/// // let repo = Repository::open(".")?;
/// // handle_new_branch(&repo, "feature-branch", Some("main"))?;
/// ```
pub fn handle_new_branch(repo: &Repository, name: &str, parent: Option<&str>) -> Result<()> {
    // Create and checkout new branch by invoking the git helper.
    git::create_new_branch(repo, name, parent)?;
    // Log the successful creation of the branch.
    info!("Created and switched to branch: {}", name);
    Ok(())
}
