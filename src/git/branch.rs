use crate::error::{GitFlowError, Result};
use git2::{BranchType, Repository};
use log::info;

/// Get the current branch name from the Git repository
///
/// # Arguments
///
/// * `repo` - A reference to the Git repository
///
/// # Returns
///
/// * `Result<String>` - Returns the current branch name as a string, or an error if it fails
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    // Get the reference to the current HEAD
    let head = repo.head()?;
    
    // Check if HEAD is pointing to a branch
    if !head.is_branch() {
        return Err(GitFlowError::Git(git2::Error::from_str(
            "HEAD is not a branch (detached HEAD state)",
        )));
    }
    
    // Get the branch name from the HEAD reference
    head.shorthand()
        .ok_or_else(|| GitFlowError::Git(git2::Error::from_str("Could not get branch name")))
        .map(String::from)
}

/// Create a new branch based on parent and switch to it
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
pub fn create_new_branch(repo: &Repository, name: &str, parent: Option<&str>) -> Result<()> {
    // Check if branch already exists
    if repo.find_branch(name, BranchType::Local).is_ok() {
        return Err(GitFlowError::Git(git2::Error::from_str(
            &format!("Branch '{}' already exists", name),
        )));
    }
    
    // Determine the parent branch name
    let parent_branch_name = match parent {
        Some(branch) => branch.to_string(),
        None => get_current_branch(repo)?,
    };
    
    // Find the parent branch
    let parent_branch = repo.find_branch(&parent_branch_name, BranchType::Local)
        .map_err(|_| GitFlowError::BranchNotFound(parent_branch_name.clone()))?;
        
    // Get the commit to branch from
    let commit = parent_branch.get().peel_to_commit()?;
    
    // Create the new branch
    repo.branch(name, &commit, false)?;
    
    // Checkout the new branch
    let obj = repo.revparse_single(&format!("refs/heads/{}", name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", name))?;
    
    // Log the branch creation and switch
    info!("Created and switched to branch: {}", name);
    Ok(())
}
