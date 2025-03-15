use crate::error::{GitFlowError, Result};
use git2::{BranchType, Commit, Repository};
use log::{debug, info};
use std::collections::{HashMap, HashSet};

/// Build a tree of branches showing parent-child relationships
pub fn get_branch_tree(repo: &Repository) -> Result<HashMap<String, Vec<String>>> {
    let mut tree = HashMap::new();
    let branches = repo.branches(Some(BranchType::Local))?;

    // First pass: collect all branches
    let mut all_branches = Vec::new();
    for branch_result in branches {
        let (branch, _) = branch_result?;
        let name = branch
            .name()?
            .ok_or_else(|| {
                GitFlowError::Git(git2::Error::from_str("Invalid UTF-8 in branch name"))
            })?
            .to_string();

        all_branches.push(name);
    }

    // Second pass: build the tree
    for branch_name in &all_branches {
        let commit = repo.revparse_single(branch_name)?.peel_to_commit()?;

        for other_branch in &all_branches {
            if branch_name == other_branch {
                continue;
            }

            let other_commit = repo.revparse_single(other_branch)?.peel_to_commit()?;

            // Check if other_branch is a direct descendant of branch_name
            if is_descendant_of(repo, &other_commit, &commit)?
                && !is_direct_parent_child(&all_branches, branch_name, other_branch, repo)?
            {
                tree.entry(branch_name.clone())
                    .or_insert_with(Vec::new)
                    .push(other_branch.clone());
            }
        }
    }

    Ok(tree)
}

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
        return Err(GitFlowError::Git(git2::Error::from_str(&format!(
            "Branch '{}' already exists",
            name
        ))));
    }

    // Determine the parent branch name
    let parent_branch_name = match parent {
        Some(branch) => branch.to_string(),
        None => get_current_branch(repo)?,
    };

    // Find the parent branch
    let parent_branch = repo
        .find_branch(&parent_branch_name, BranchType::Local)
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

/// Check if there's a more direct parent between potential parent and child
pub fn is_direct_parent_child(
    all_branches: &[String],
    parent: &str,
    child: &str,
    repo: &Repository,
) -> Result<bool> {
    let parent_commit = repo.revparse_single(parent)?.peel_to_commit()?;
    let child_commit = repo.revparse_single(child)?.peel_to_commit()?;

    // Check all other branches to see if any are between parent and child
    for other in all_branches {
        if other != parent && other != child {
            let other_commit = repo.revparse_single(other)?.peel_to_commit()?;

            // If other is between parent and child, this is not a direct relationship
            if is_descendant_of(repo, &other_commit, &parent_commit)?
                && is_descendant_of(repo, &child_commit, &other_commit)?
            {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Check if commit is a descendant of potential_ancestor
pub fn is_descendant_of(
    repo: &Repository,
    commit: &Commit,
    potential_ancestor: &Commit,
) -> Result<bool> {
    if commit.id() == potential_ancestor.id() {
        return Ok(false); // Same commit, not a descendant
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    // Push the commit's ancestors
    revwalk.push(commit.id())?;

    // Look for the potential ancestor
    for ancestor_id in revwalk {
        let ancestor_id = ancestor_id?;
        if ancestor_id == potential_ancestor.id() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Find root branches (those without parents)
pub fn find_root_branches(branch_tree: &HashMap<String, Vec<String>>) -> Vec<String> {
    let all_children: HashSet<String> = branch_tree.values().flatten().cloned().collect();

    branch_tree
        .keys()
        .filter(|branch| !all_children.contains(*branch))
        .cloned()
        .collect()
}

/// Checkout a branch by name
pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    debug!("Checked out branch: {}", branch_name);
    Ok(())
}
