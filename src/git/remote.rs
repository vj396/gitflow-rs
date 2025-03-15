use crate::error::{GitFlowError, Result};
use git2::{PushOptions, Repository};
use log::info;
use std::path::PathBuf;
use url::Url;

/// Extract owner and repo name from the GitHub remote URL
pub fn get_repo_info(repo: &Repository) -> Result<(String, String)> {
    let remote = repo
        .find_remote("origin")
        .map_err(|_| GitFlowError::RemoteNotFound("origin".to_string()))?;

    let url = remote
        .url()
        .ok_or_else(|| GitFlowError::Config("Remote URL not found".to_string()))?;

    parse_github_url(url)
}

/// Parse a GitHub URL to extract owner and repository name
pub fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle various GitHub URL formats

    // For SSH URLs: git@github.com:owner/repo.git
    if url.starts_with("git@github.com:") {
        let path = url.trim_start_matches("git@github.com:");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0].to_string();
            let repo = parts[1].trim_end_matches(".git").to_string();
            return Ok((owner, repo));
        }
    }

    // For HTTPS URLs: https://github.com/owner/repo.git
    if url.contains("github.com") {
        let parsed_url = Url::parse(url)?;
        let path_segments: Vec<&str> = parsed_url
            .path_segments()
            .ok_or_else(|| GitFlowError::Config("Invalid GitHub URL".to_string()))?
            .collect();

        if path_segments.len() >= 2 {
            let owner = path_segments[0].to_string();
            let repo = path_segments[1].trim_end_matches(".git").to_string();
            return Ok((owner, repo));
        }
    }

    Err(GitFlowError::Config(format!(
        "Could not parse GitHub URL: {}",
        url
    )))
}

/// Push the current branch to the remote repository
pub fn push_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let remote_name = "origin";
    let mut remote = repo.find_remote(remote_name)?;

    // Check if remote exists
    if remote.url().is_none() {
        return Err(GitFlowError::RemoteNotFound(remote_name.to_string()));
    }

    // Create a push refspec
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    // Configure push options with authentication
    let mut push_options = setup_push_options()?;

    // Push the branch
    remote.push(&[&refspec], Some(&mut push_options))?;

    info!("Pushed branch {} to remote {}", branch_name, remote_name);
    Ok(())
}

/// Setup push options with authentication
fn setup_push_options() -> Result<PushOptions<'static>> {
    let mut push_options = PushOptions::new();

    // Use credentials from environment if available
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        let username = username_from_url.unwrap_or("git");

        // Try SSH key authentication
        let ssh_key_path = dirs::home_dir()
            .map(|h| h.join(".ssh/id_rsa"))
            .unwrap_or_else(|| PathBuf::from("~/.ssh/id_rsa"));

        git2::Cred::ssh_key(username, None, &ssh_key_path, None)
    });

    push_options.remote_callbacks(callbacks);
    Ok(push_options)
}
