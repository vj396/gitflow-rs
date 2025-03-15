use crate::configuration::Config;
use crate::error::{GitFlowError, Result};
use crate::git;
use crate::github;
use crate::utils::{prompt_confirmation, prompt_input};
use git2::Repository;
use log::info;
use tokio::runtime::Runtime;

/// Handle the 'sync' command to commit changes, push branch, and create PR
pub fn handle_sync(
    repo: &Repository,
    title_opt: Option<&str>,
    yes: bool,
    base_opt: Option<&str>,
) -> Result<()> {
    // Get current branch name
    let current_branch = git::get_current_branch(repo)?;

    // Check if there are uncommitted changes
    let status = git::get_repo_status(repo, true)?;

    if !status.is_empty() {
        info!("Uncommitted changes detected:");

        for entry in &status {
            info!(
                "  {} {}",
                git::format_status_entry(entry.status),
                entry.path
            );
        }

        if !yes && !prompt_confirmation("Do you want to commit these changes?")? {
            return Err(GitFlowError::Aborted(
                "Sync operation cancelled".to_string(),
            ));
        }

        let message = prompt_input("Enter commit message")?;
        git::commit_changes(repo, &message)?;
    }

    // Load config to get default base branch
    let config = Config::load()?;

    // Determine base branch for PR
    let base_branch = match base_opt {
        Some(base) => base.to_string(),
        None => git::get_parent_branch(repo, &current_branch, &config.default_base_branch)?,
    };

    // Push the branch to remote
    info!("Pushing branch {} to remote...", current_branch);
    git::push_branch(repo, &current_branch)?;

    // Determine PR title
    let title = match title_opt {
        Some(t) => t.to_string(),
        None => git::get_last_commit_summary(repo)?,
    };

    // Get owner and repo name
    let (owner, repo_name) = git::get_repo_info(repo)?;

    // Create PR
    info!("Creating pull request...");

    // We'll use tokio runtime to run the async function
    let rt = Runtime::new()?;

    // Check if PR already exists
    let existing_pr = rt.block_on(github::check_existing_pr(
        &owner,
        &repo_name,
        &current_branch,
    ))?;

    let pr_info = if let Some(pr) = existing_pr {
        info!("Pull request already exists: {}", pr.url);
        info!("PR #{}: {}", pr.number, pr.title);
        pr
    } else {
        // Create new PR
        let pr = rt.block_on(github::create_pull_request(
            &owner,
            &repo_name,
            &current_branch,
            &base_branch,
            &title,
        ))?;

        info!("Pull request created: {}", pr.url);
        info!("PR #{}: {}", pr.number, pr.title);
        pr
    };

    // Save PR URL in config
    let mut config = Config::load()?;
    config.add_pr(current_branch, pr_info)?;

    Ok(())
}
