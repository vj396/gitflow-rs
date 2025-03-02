use crate::error::Result;
use crate::git;
use git2::Repository;
use log::info;

/// Handle the 'new' command to create a new branch
///
/// # Arguments
///
/// * `repo` - A reference to the Git repository
/// * `name` - The name of the new branch to create
/// * `parent` - An optional parent branch name to base the new branch on
///
/// # Returns
///
/// * `Result<()>` - Returns an empty Ok result on success, or an error on failure
pub fn handle_new_branch(repo: &Repository, name: &str, parent: Option<&str>) -> Result<()> {
    // Create and checkout new branch
    git::create_new_branch(repo, name, parent)?;

    info!("Created and switched to branch: {}", name);
    Ok(())
}
