use crate::configuration::PrInfo;
use crate::error::{GitFlowError, Result};
use log::debug;
use std::env;
use std::fs;
use std::path::Path;

/// Create a pull request on GitHub
pub async fn create_pull_request(
    owner: &str,
    repo_name: &str,
    branch: &str,
    base: &str,
    title: &str,
) -> Result<PrInfo> {
    // Get GitHub token
    let token = env::var("GITHUB_TOKEN")
        .map_err(|_| GitFlowError::Environment("GITHUB_TOKEN environment variable not set".to_string()))?;
    
    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(token)
        .build()
        .map_err(|e| GitFlowError::GitHub(e))?;
    
    // Try to read PR template
    let template_path = Path::new(".github/pull_request_template.md");
    let body = if template_path.exists() {
        fs::read_to_string(template_path).unwrap_or_default()
    } else {
        debug!("No PR template found at {}", template_path.display());
        String::new()
    };
    
    debug!("Creating PR: {} -> {} ({})", branch, base, title);
    
    // Create the PR
    let pr = octocrab
        .pulls(owner, repo_name)
        .create(title, branch, base)
        .body(body)
        .send()
        .await?;
    
    let pr_info = PrInfo {
        url: pr.html_url.unwrap().to_string(),
        number: pr.number,
        title: pr.title.unwrap_or_else(|| title.to_string()),
        created_at: pr.created_at.map(|d| d.to_string()).unwrap_or_else(|| "Unknown".to_string()),
    };
    
    Ok(pr_info)
}

/// Check if a pull request already exists for a branch
pub async fn check_existing_pr(
    owner: &str,
    repo_name: &str,
    branch: &str,
) -> Result<Option<PrInfo>> {
    // Get GitHub token
    let token = env::var("GITHUB_TOKEN")
        .map_err(|_| GitFlowError::Environment("GITHUB_TOKEN environment variable not set".to_string()))?;
    
    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(token)
        .build()
        .map_err(|e| GitFlowError::GitHub(e))?;
    
    // Get open PRs for the branch
    let prs = octocrab
        .pulls(owner, repo_name)
        .list()
        .state(octocrab::params::State::Open)
        .head(format!("{}:{}", owner, branch))
        .send()
        .await?;
    
    // Check if there's a matching PR
    for pr in prs {
        if pr.head.ref_field == branch {
            return Ok(Some(PrInfo {
                url: pr.html_url.unwrap().to_string(),
                number: pr.number,
                title: pr.title.unwrap_or_else(|| "Unknown".to_string()),
                created_at: pr.created_at.map(|d| d.to_string()).unwrap_or_else(|| "Unknown".to_string()),
            }));
        }
    }
    
    Ok(None)
}