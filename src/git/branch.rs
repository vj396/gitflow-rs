//! Module for Git branch operations.
//!
//! This module provides functions to manage branches including detecting branch relationships
//! using various strategies (commit history, branch creation time, default root, or manual settings),
//! creating new branches, checking out branches, and other utilities such as finding a branch's
//! commit history and parent-child relationships.
//!
//! # Details
//! Detailed documentation is provided here to facilitate maintenance and clarity on branch operations.

use crate::configuration::Config;
use crate::error::{GitFlowError, Result};
use git2::{BranchType, Commit, Repository};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Defines the strategy to use for detecting branch relationships
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BranchRelationStrategy {
    /// Use commit history to determine relationships (original method)
    CommitHistory,
    /// Use branch creation timestamps
    CreationTime,
    /// Consider main/master as parent of all other branches
    DefaultRoot,
    /// Use explicit configuration
    Manual,
}

impl Default for BranchRelationStrategy {
    fn default() -> Self {
        BranchRelationStrategy::CommitHistory
    }
}

/// Get the current branch name from the Git repository
///
/// # Arguments
///
/// * `repo` - A reference to the Repository instance.
///
/// # Returns
///
/// * `Result<String>` - The current branch name on success or an error if not in a branch.
///
/// # Examples
/// ```rust
/// // Assuming 'repo' is a valid &Repository:
/// let branch = get_current_branch(&repo)?;
/// println!("Current branch: {}", branch);
/// ```
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;

    if (!head.is_branch()) {
        return Err(GitFlowError::Git(git2::Error::from_str(
            "HEAD is not a branch (detached HEAD state)",
        )));
    }

    head.shorthand()
        .ok_or_else(|| GitFlowError::Git(git2::Error::from_str("Could not get branch name")))
        .map(String::from)
}

/// Create a new branch based on parent and switch to it
///
/// # Arguments
///
/// * `repo`   - The Git repository.
/// * `name`   - The name for the new branch.
/// * `parent` - An optional parent branch name.
///
/// # Returns
///
/// * `Result<()>` - Ok on success, or an error if the branch already exists or if creation fails.
///
/// # Examples
/// ```rust
/// // Create a new branch "feature" based on the current branch:
/// create_new_branch(&repo, "feature", None)?;
/// ```
pub fn create_new_branch(repo: &Repository, name: &str, parent: Option<&str>) -> Result<()> {
    // Check if branch already exists.
    if repo.find_branch(name, BranchType::Local).is_ok() {
        return Err(GitFlowError::Git(git2::Error::from_str(&format!(
            "Branch '{}' already exists",
            name
        ))));
    }

    // Determine the parent branch name.
    let parent_branch_name = match parent {
        Some(branch) => branch.to_string(),
        None => get_current_branch(repo)?,
    };

    // Locate the parent branch.
    let parent_branch = repo
        .find_branch(&parent_branch_name, BranchType::Local)
        .map_err(|_| GitFlowError::BranchNotFound(parent_branch_name.clone()))?;

    // Get the commit to base the new branch on.
    let commit = parent_branch.get().peel_to_commit()?;

    // Create the new branch.
    repo.branch(name, &commit, false)?;

    // Checkout and set HEAD to the new branch.
    let obj = repo.revparse_single(&format!("refs/heads/{}", name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", name))?;

    info!("Created and switched to branch: {}", name);
    Ok(())
}

/// Build a tree of branches showing parent-child relationships
///
/// # Arguments
///
/// * `repo`     - The repository reference.
/// * `strategy` - The branch relation strategy to use.
/// * `config`   - Reference to the configuration settings.
///
/// # Returns
///
/// * `Result<HashMap<String, Vec<String>>>` - A mapping from parent branch names to their child branches.
///
/// # Examples
/// ```rust
/// // Build the branch tree using the commit history strategy:
/// let tree = get_branch_tree(&repo, BranchRelationStrategy::CommitHistory, &config)?;
/// ```
pub fn get_branch_tree(
    repo: &Repository,
    strategy: BranchRelationStrategy,
    config: &Config,
) -> Result<HashMap<String, Vec<String>>> {
    match strategy {
        BranchRelationStrategy::CommitHistory => get_branch_tree_by_history(repo),
        BranchRelationStrategy::CreationTime => get_branch_tree_by_creation_time(repo),
        BranchRelationStrategy::DefaultRoot => {
            get_branch_tree_with_default_root(repo, &config.default_base_branch)
        }
        BranchRelationStrategy::Manual => Ok(config.branch_relationships.clone()),
    }
}

/// Build branch tree using commit history (original method)
///
/// # Arguments
/// * repo - Reference to the Git repository.
/// 
/// # Returns
/// A Result containing a HashMap mapping parent branches to their child branch lists.
fn get_branch_tree_by_history(repo: &Repository) -> Result<HashMap<String, Vec<String>>> {
    let mut tree = HashMap::new();
    let branches = repo.branches(Some(BranchType::Local))?;

    // First pass: collect all branch names.
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

    // Second pass: build the parent-child relationships.
    for branch_name in &all_branches {
        let commit = repo.revparse_single(branch_name)?.peel_to_commit()?;
        for other_branch in &all_branches {
            if branch_name == other_branch {
                continue;
            }
            let other_commit = repo.revparse_single(other_branch)?.peel_to_commit()?;
            // Determine if 'other_branch' is a descendant of 'branch_name'
            if is_descendant_of(repo, &other_commit, &commit)?
                && !is_direct_parent_child(&all_branches, branch_name, other_branch, repo)?
            {
                tree.entry(branch_name.clone())
                    .or_insert_with(Vec::new)
                    .push(other_branch.clone());
            }
        }
    }

    debug!("Branch tree by history: {:?}", tree);
    Ok(tree)
}

/// Build branch tree based on branch creation times
///
/// # Arguments
/// * repo - Reference to the Git repository.
/// 
/// # Returns
/// A Result containing a HashMap mapping parent branches to their child branch lists.
fn get_branch_tree_by_creation_time(repo: &Repository) -> Result<HashMap<String, Vec<String>>> {
    let mut tree = HashMap::new();
    let branches = repo.branches(Some(BranchType::Local))?;

    // Get all branches with an approximation of their creation time (first commit time).
    let mut branch_times = Vec::new();
    for branch_result in branches {
        let (branch, _) = branch_result?;
        let name = branch.name()?.unwrap_or("").to_string();
        if let Ok(commit) = get_first_commit_on_branch(repo, &name) {
            let time = commit.time().seconds();
            branch_times.push((name, time));
        }
    }

    // Sort branches by creation time (oldest first).
    branch_times.sort_by(|a, b| a.1.cmp(&b.1));
    debug!("Branches by creation time: {:?}", branch_times);

    // Determine parent-child relationships based on time proximity.
    if !branch_times.is_empty() {
        for i in 1..branch_times.len() {
            let child = branch_times[i].0.clone();
            let child_time = branch_times[i].1;
            for j in (0..i).rev() {
                let potential_parent = branch_times[j].0.clone();
                let parent_time = branch_times[j].1;
                if child_time - parent_time < 60 * 60 * 24 * 30 && // within 30 days
                   are_branches_related(repo, &potential_parent, &child)?
                {
                    tree.entry(potential_parent)
                        .or_insert_with(Vec::new)
                        .push(child);
                    break;
                }
            }
        }
    }

    debug!("Branch tree by creation time: {:?}", tree);
    Ok(tree)
}

/// Build branch tree considering the default branch as the root of all others
///
/// # Arguments
/// * repo - Reference to the Git repository.
/// * default_branch - The branch to use as the root for others.
/// 
/// # Returns
/// A Result containing a HashMap with the default branch mapped to all other branches.
fn get_branch_tree_with_default_root(
    repo: &Repository,
    default_branch: &str,
) -> Result<HashMap<String, Vec<String>>> {
    let mut tree = HashMap::new();
    let branches = repo.branches(Some(BranchType::Local))?;

    // Verify the default branch exists.
    if repo.find_branch(default_branch, BranchType::Local).is_err() {
        return Ok(tree);
    }

    // Collect branches other than the default.
    let mut other_branches = Vec::new();
    for branch_result in branches {
        let (branch, _) = branch_result?;
        let name = branch.name()?.unwrap_or("").to_string();
        if name != default_branch {
            other_branches.push(name);
        }
    }

    // Set the default branch as the parent for all other branches.
    if !other_branches.is_empty() {
        tree.insert(default_branch.to_string(), other_branches);
    }

    debug!("Branch tree with default root: {:?}", tree);
    Ok(tree)
}

/// Check if there's a more direct parent between the potential parent and child branches.
///
/// # Arguments
/// * all_branches - Slice of all branch names.
/// * parent - The candidate parent branch name.
/// * child - The candidate child branch name.
/// * repo - The repository reference.
/// 
/// # Returns
/// A Result with true if it's a direct parent-child relationship, false otherwise.
pub fn is_direct_parent_child(
    all_branches: &[String],
    parent: &str,
    child: &str,
    repo: &Repository,
) -> Result<bool> {
    let parent_commit = repo.revparse_single(parent)?.peel_to_commit()?;
    let child_commit = repo.revparse_single(child)?.peel_to_commit()?;

    // Check if any other branch is between parent and child.
    for other in all_branches {
        if other != parent && other != child {
            let other_commit = repo.revparse_single(other)?.peel_to_commit()?;
            if is_descendant_of(repo, &other_commit, &parent_commit)?
                && is_descendant_of(repo, &child_commit, &other_commit)?
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Check if 'commit' is a descendant of 'potential_ancestor'
///
/// # Arguments
/// * repo - The repository reference.
/// * commit - The commit to evaluate.
/// * potential_ancestor - The commit considered as an ancestor candidate.
/// 
/// # Returns
/// A Result with true if 'commit' is a descendant, false otherwise.
pub fn is_descendant_of(
    repo: &Repository,
    commit: &Commit,
    potential_ancestor: &Commit,
) -> Result<bool> {
    if commit.id() == potential_ancestor.id() {
        return Ok(false); // The commits are identical.
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push(commit.id())?;

    for ancestor_id in revwalk {
        let ancestor_id = ancestor_id?;
        if ancestor_id == potential_ancestor.id() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Check if two branches share some commit history
///
/// # Arguments
/// * repo - The repository reference.
/// * branch1 - The first branch name.
/// * branch2 - The second branch name.
/// 
/// # Returns
/// A Result with true if the branches are related, false otherwise.
fn are_branches_related(repo: &Repository, branch1: &str, branch2: &str) -> Result<bool> {
    let commit1 = repo.revparse_single(branch1)?.peel_to_commit()?;
    let commit2 = repo.revparse_single(branch2)?.peel_to_commit()?;

    // Check if one commit is an ancestor of the other.
    if is_descendant_of(repo, &commit1, &commit2)? || is_descendant_of(repo, &commit2, &commit1)? {
        return Ok(true);
    }

    // Use merge-base to determine if there is any common ancestry.
    match repo.merge_base(commit1.id(), commit2.id()) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Get the first commit on a branch (approximates branch creation time)
///
/// # Arguments
/// * repo - The repository reference.
/// * branch_name - The branch name in question.
///
/// # Returns
/// A Result with the first commit found on the branch.
fn get_first_commit_on_branch<'repo>(
    repo: &'repo Repository,
    branch_name: &str,
) -> Result<Commit<'repo>> {
    let commit = repo.revparse_single(branch_name)?.peel_to_commit()?;
    Ok(commit)
}

/// Get the parent branch of the current branch using history and creation time strategies
///
/// # Arguments
///
/// * `repo`           - The repository.
/// * `current_branch` - The current branch name.
/// * `default_base`   - The default branch to use if no parent is found.
///
/// # Returns
///
/// * `Result<String>` - The parent branch name.
///
/// # Examples
/// ```rust
/// // Retrieve the parent branch for "feature" with default "main":
/// let parent = get_parent_branch(&repo, "feature", "main")?;
/// ```
pub fn get_parent_branch(
    repo: &Repository,
    current_branch: &str,
    default_base: &str,
) -> Result<String> {
    let branch_tree = get_branch_tree_by_history(repo)?;
    for (parent, children) in &branch_tree {
        if children.contains(&current_branch.to_string()) {
            return Ok(parent.clone());
        }
    }
    let branch_tree_by_time = get_branch_tree_by_creation_time(repo)?;
    for (parent, children) in &branch_tree_by_time {
        if children.contains(&current_branch.to_string()) {
            return Ok(parent.clone());
        }
    }
    Ok(default_base.to_string())
}

/// Checkout a branch by its name
///
/// # Arguments
///
/// * `repo`        - The repository.
/// * `branch_name` - The target branch name.
///
/// # Returns
///
/// * `Result<()>` - Ok if the checkout was successful.
///
/// # Examples
/// ```rust
/// // Checkout branch "develop":
/// checkout_branch(&repo, "develop")?;
/// ```
pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    debug!("Checked out branch: {}", branch_name);
    Ok(())
}

/// Find root branches (those without any parent branch)
///
/// # Arguments
///
/// * `branch_tree` - A branch tree mapping parents to their children.
///
/// # Returns
///
/// * `Vec<String>` - A list of branch names that are roots.
pub fn find_root_branches(branch_tree: &HashMap<String, Vec<String>>) -> Vec<String> {
    let all_children: HashSet<String> = branch_tree.values().flatten().cloned().collect();
    branch_tree
        .keys()
        .filter(|branch| !all_children.contains(*branch))
        .cloned()
        .collect()
}

/// Get the latest commit for a branch
///
/// # Arguments
///
/// * `repo`        - The repository.
/// * `branch_name` - The branch name.
///
/// # Returns
///
/// * `Result<Commit>` - The latest commit on the branch.
///
/// # Examples
/// ```rust
/// // Retrieve the latest commit of branch "main":
/// let commit = get_branch_commit(&repo, "main")?;
/// ```
pub fn get_branch_commit<'repo>(
    repo: &'repo Repository,
    branch_name: &str,
) -> Result<Commit<'repo>> {
    let obj = repo.revparse_single(branch_name)?;
    obj.peel_to_commit().map_err(GitFlowError::Git)
}
