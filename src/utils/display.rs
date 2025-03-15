//! Module for displaying output.
//!
//! This module provides utilities such as prompting the user for confirmation,
//! formatting branch names and PR links, and printing the branch hierarchy.
//!
//! # Details
//! Detailed examples and descriptions are provided to facilitate future code maintenance.

use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::io::{self, Write};

/// Prompt the user for confirmation with a yes/no question
///
/// # Arguments
/// * `message` - The prompt message to display.
///
/// # Returns
/// * `io::Result<bool>` - Returns true if the user confirms with 'y', false otherwise.
///
/// # Examples
/// ```rust
/// // Example:
/// // if prompt_confirmation("Proceed with action?")? { ... }
/// ```
pub fn prompt_confirmation(message: &str) -> io::Result<bool> {
    print!("{} [y/N]: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// Prompt the user for input with a message
pub fn prompt_input(message: &str) -> io::Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Format a branch name with color based on whether it's the current branch
///
/// # Arguments
/// * `branch_name` - The branch name to format.
/// * `is_current`  - Flag indicating if this is the current branch.
///
/// # Returns
/// * `ColoredString` - The formatted branch name with color.
///
/// # Examples
/// ```rust
/// // Example:
/// // let formatted = format_branch_name("feature", true);
/// ```
pub fn format_branch_name(branch_name: &str, is_current: bool) -> ColoredString {
    if is_current {
        format!("* {}", branch_name).green()
    } else {
        format!("  {}", branch_name).normal()
    }
}

/// Format a PR link for display
///
/// # Arguments
/// * `number` - The pull request number.
/// * `url`    - The URL of the pull request.
///
/// # Returns
/// * `ColoredString` - The formatted PR link.
///
/// # Examples
/// ```rust
/// // Example:
/// // let pr = format_pr_link(42, "http://example.com/pr/42");
/// ```
pub fn format_pr_link(number: u64, url: &str) -> ColoredString {
    format!(" [PR #{}]({})", number, url).blue()
}

/// Print the branch tree as a hierarchy
///
/// # Arguments
/// * `tree`            - A mapping of parent branch names to their child branches.
/// * `root_branches`   - A list of branches with no parent.
/// * `current_branch`  - The current checked-out branch name.
/// * `pr_info`         - A mapping of branch names to PR information tuples.
/// * `commit_messages` - A mapping of branch names to their first commit message line.
///
/// # Returns
/// * None
///
/// # Examples
/// ```rust
/// // Example:
/// // print_branch_hierarchy(&branch_tree, &roots, "main", &pr_info, &commit_msgs);
/// ```
pub fn print_branch_hierarchy(
    tree: &HashMap<String, Vec<String>>,
    root_branches: &[String],
    current_branch: &str,
    pr_info: &HashMap<String, (u64, String)>,
    commit_messages: &HashMap<String, String>,
) {
    // Helper function to print branch tree recursively
    fn print_branch_tree(
        branch: &str,
        tree: &HashMap<String, Vec<String>>,
        current_branch: &str,
        pr_info: &HashMap<String, (u64, String)>,
        commit_messages: &HashMap<String, String>,
        prefix: &str,
        is_last: bool,
    ) {
        // Format branch name with PR link if available
        let branch_display = format_branch_name(branch, branch == current_branch);

        let pr_display = if let Some((number, url)) = pr_info.get(branch) {
            format_pr_link(*number, url)
        } else {
            "".normal()
        };

        // Get commit message if available
        let commit_display = if let Some(message) = commit_messages.get(branch) {
            format!(" \"{}\"", message).yellow()
        } else {
            "".normal()
        };

        // Format branch line
        let branch_symbol = if is_last { "└── " } else { "├── " };

        println!(
            "{}{}{}{}{}",
            prefix, branch_symbol, branch_display, pr_display, commit_display
        );

        // Process children
        if let Some(children) = tree.get(branch) {
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            let count = children.len();
            for (i, child) in children.iter().enumerate() {
                print_branch_tree(
                    child,
                    tree,
                    current_branch,
                    pr_info,
                    commit_messages,
                    &new_prefix,
                    i == count - 1,
                );
            }
        }
    }

    // Print the tree starting from root branches
    let count = root_branches.len();
    for (i, branch) in root_branches.iter().enumerate() {
        print_branch_tree(
            branch,
            tree,
            current_branch,
            pr_info,
            commit_messages,
            "",
            i == count - 1,
        );
    }
}
