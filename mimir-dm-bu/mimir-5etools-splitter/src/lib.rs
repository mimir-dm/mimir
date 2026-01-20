//! 5etools Repository Splitter
//!
//! This crate processes 5etools repositories and splits them into individual
//! book archives for import into Mimir.

#![warn(missing_docs)]

/// Archive creation utilities.
pub mod archive;
/// Content collection from 5etools data.
pub mod collector;
/// Content filtering utilities.
pub mod filter;
/// Image processing and extraction.
pub mod images;
/// Input source handling (local paths and GitHub URLs).
pub mod input;
/// Load testing for archive validation.
pub mod load_tester;
/// Magic item variant expansion.
pub mod magic_variants;
/// Parallel processing utilities.
pub mod parallel;
/// Book metadata parsing.
pub mod parser;
/// Repository setup and cloning.
pub mod repo;
/// SRD content collection.
pub mod srd_collector;
/// SRD content filtering.
pub mod srd_filter;

use anyhow::Result;
use std::path::Path;

pub use input::InputSource;
pub use parser::Book;

/// Process a 5etools repository and split it into book archives
pub async fn split_repository(input: InputSource, output_dir: &Path) -> Result<SplitResults> {
    // Setup repository (clone if needed)
    let repo_path = repo::setup_repository(input).await?;

    // Load all books
    let books = parser::load_all_books(&repo_path)?;

    // Process books in parallel
    let results = parallel::process_all_books(books, &repo_path, output_dir)?;

    Ok(results)
}

/// Extract SRD content from a 5etools repository
pub async fn extract_srd(input: InputSource, output_dir: &Path) -> Result<SrdResults> {
    use crate::srd_collector::collect_srd_content;

    // Setup repository (clone if needed)
    let repo_path = repo::setup_repository(input).await?;

    // Collect all SRD content
    let content = collect_srd_content(&repo_path)?;

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Generate summary
    let summary = srd_collector::generate_srd_summary(&content);

    // Create archive
    let archive_path = output_dir.join("srd.tar.gz");
    archive::create_tar_gz(&content.files, &archive_path)?;

    Ok(SrdResults {
        archive_path: archive_path.to_string_lossy().to_string(),
        total_items: content.metadata.total_items,
        content_summary: content.metadata.content_summary,
        summary,
    })
}

/// Results from splitting a 5etools repository into book archives.
#[derive(Debug)]
pub struct SplitResults {
    /// Book IDs that were successfully processed.
    pub successful: Vec<String>,
    /// Failed books as (book_id, error_message) tuples.
    pub failed: Vec<(String, String)>,
    /// Total number of books processed.
    pub total_processed: usize,
}

/// Results from extracting SRD content from a repository.
#[derive(Debug)]
pub struct SrdResults {
    /// Path to the generated archive file.
    pub archive_path: String,
    /// Total number of SRD items extracted.
    pub total_items: usize,
    /// Count of items by content type.
    pub content_summary: std::collections::HashMap<String, usize>,
    /// Human-readable summary of the extraction.
    pub summary: String,
}
