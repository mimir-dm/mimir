use crate::input::InputSource;
use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Setup repository - either use local path or clone from GitHub
pub async fn setup_repository(input: InputSource) -> Result<PathBuf> {
    match input {
        InputSource::LocalPath(path) => {
            validate_repository_structure(&path)?;
            Ok(path)
        }
        InputSource::GitHub { url, reference } => {
            clone_repository(&url, reference.as_deref()).await
        }
    }
}

/// Validate that the directory has expected 5etools structure
fn validate_repository_structure(path: &Path) -> Result<()> {
    // Check for essential directories
    let data_dir = path.join("data");
    if !data_dir.exists() {
        return Err(anyhow!(
            "Invalid 5etools repository: missing 'data' directory at {:?}",
            path
        ));
    }

    // Check for books.json
    let books_file = data_dir.join("books.json");
    if !books_file.exists() {
        return Err(anyhow!(
            "Invalid 5etools repository: missing 'data/books.json' at {:?}",
            path
        ));
    }

    Ok(())
}

/// Clone repository from GitHub to a temporary directory using git command
async fn clone_repository(url: &str, reference: Option<&str>) -> Result<PathBuf> {
    // Create temporary directory
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let repo_path = temp_dir.path().to_path_buf();

    // Setup progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Cloning {}", url));

    // Build git clone command
    let mut cmd = Command::new("git");
    cmd.arg("clone");

    // Add branch/tag if specified
    if let Some(ref_str) = reference {
        cmd.arg("--branch").arg(ref_str);
    }

    // Add depth to speed up clone
    cmd.arg("--depth").arg("1");

    // Add URL and destination
    cmd.arg(url).arg(&repo_path);

    // Execute clone
    let output = cmd.output().context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git clone failed: {}", stderr));
    }

    pb.finish_with_message("Clone complete");

    // Clone image repository if applicable
    let input_source = InputSource::parse(url)?;
    if let Some(img_url) = input_source.get_image_repo_url() {
        clone_image_repository(&img_url, &repo_path).await?;
    }

    // Validate structure
    validate_repository_structure(&repo_path)?;

    // Keep directory persistent
    std::mem::forget(temp_dir);
    Ok(repo_path)
}

/// Clone image repository into img/ subdirectory
async fn clone_image_repository(img_url: &str, parent_path: &Path) -> Result<()> {
    let img_path = parent_path.join("img");

    // Skip if img directory already exists
    if img_path.exists() {
        return Ok(());
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Cloning images from {}", img_url));

    // Parse URL and reference
    let parts: Vec<&str> = img_url.splitn(2, '@').collect();
    let url = parts[0];

    // Build git clone command
    let mut cmd = Command::new("git");
    cmd.arg("clone");

    // Add branch/tag if specified
    if let Some(ref_str) = parts.get(1) {
        cmd.arg("--branch").arg(ref_str);
    }

    // Add depth to speed up clone
    cmd.arg("--depth").arg("1");

    // Add URL and destination
    cmd.arg(url).arg(&img_path);

    // Execute clone
    let output = cmd
        .output()
        .context("Failed to execute git clone for images")?;

    if !output.status.success() {
        // Images are optional, so just warn
        pb.finish_with_message("Image clone failed (optional)");
        return Ok(());
    }

    pb.finish_with_message("Image clone complete");
    Ok(())
}
