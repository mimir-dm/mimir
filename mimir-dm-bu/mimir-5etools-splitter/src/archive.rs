use crate::collector::BookContent;
use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use tar::Builder;

/// Create a tar.gz archive from book content
pub fn create_archive(content: &BookContent, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .context(format!("Failed to create archive file: {:?}", output_path))?;

    let gz = GzEncoder::new(file, Compression::default());
    let mut archive = Builder::new(gz);

    // Create base directory name
    let base_dir = &content.book.id;

    // Add all files to the archive
    for (path, data) in &content.files {
        // Create full path with base directory
        let full_path = format!("{}/{}", base_dir, path);

        // Create a header for the file
        let mut header = tar::Header::new_gnu();
        header.set_path(&full_path)?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        // Append the file to the archive
        archive
            .append(&header, data.as_slice())
            .context(format!("Failed to add {} to archive", full_path))?;
    }

    // Finish the archive
    archive
        .into_inner()
        .context("Failed to finalize archive")?
        .finish()
        .context("Failed to compress archive")?;

    Ok(())
}

/// Create a tar.gz archive from a HashMap of files
pub fn create_tar_gz(files: &HashMap<String, Vec<u8>>, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .context(format!("Failed to create archive file: {:?}", output_path))?;

    let gz = GzEncoder::new(file, Compression::default());
    let mut archive = Builder::new(gz);

    // Base directory name for SRD content
    let base_dir = "SRD";

    // Add all files to the archive
    for (path, data) in files {
        // Create full path with base directory
        let full_path = format!("{}/{}", base_dir, path);

        // Create a header for the file
        let mut header = tar::Header::new_gnu();
        header.set_path(&full_path)?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        // Append the file to the archive
        archive
            .append(&header, data.as_slice())
            .context(format!("Failed to add {} to archive", full_path))?;
    }

    // Finish the archive
    archive
        .into_inner()
        .context("Failed to finalize archive")?
        .finish()
        .context("Failed to compress archive")?;

    Ok(())
}
