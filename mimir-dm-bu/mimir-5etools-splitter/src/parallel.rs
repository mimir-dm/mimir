use crate::archive;
use crate::collector;
use crate::parser::Book;
use crate::SplitResults;
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

/// Process all books in parallel
pub fn process_all_books(
    books: Vec<Book>,
    repo_path: &Path,
    output_dir: &Path,
) -> Result<SplitResults> {
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;

    // Setup progress bars
    let multi_progress = Arc::new(MultiProgress::new());
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-");

    // Create main progress bar
    let main_pb = multi_progress.add(ProgressBar::new(books.len() as u64));
    main_pb.set_style(style.clone());
    main_pb.set_message("Processing books");

    // Process books in parallel
    let results: Vec<(String, Result<()>)> = books
        .par_iter()
        .map(|book| {
            let book_id = book.id.clone();
            let result = process_single_book(book, repo_path, output_dir, &multi_progress, &style);
            main_pb.inc(1);
            (book_id, result)
        })
        .collect();

    main_pb.finish_with_message("All books processed");

    // Collect results
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (book_id, result) in results {
        match result {
            Ok(()) => successful.push(book_id),
            Err(e) => failed.push((book_id, e.to_string())),
        }
    }

    Ok(SplitResults {
        total_processed: books.len(),
        successful,
        failed,
    })
}

/// Process a single book
fn process_single_book(
    book: &Book,
    repo_path: &Path,
    output_dir: &Path,
    multi_progress: &Arc<MultiProgress>,
    style: &ProgressStyle,
) -> Result<()> {
    // Create progress bar for this book
    let pb = multi_progress.add(ProgressBar::new(4));
    pb.set_style(style.clone());
    pb.set_message(format!("Processing {}", book.name));

    // Collect content
    pb.set_message(format!("{}: Collecting content", book.id));
    let content = collector::collect_book_content(book, repo_path)?;
    pb.inc(1);

    // Create archive path
    let archive_name = format!("{}.tar.gz", book.id.to_lowercase());
    let archive_path = output_dir.join(archive_name);

    // Create archive
    pb.set_message(format!("{}: Creating archive", book.id));
    archive::create_archive(&content, &archive_path)?;
    pb.inc(1);

    // Verify archive was created
    pb.set_message(format!("{}: Verifying", book.id));
    if !archive_path.exists() {
        return Err(anyhow::anyhow!(
            "Archive was not created: {:?}",
            archive_path
        ));
    }
    pb.inc(1);

    // Get file size for reporting
    let metadata = std::fs::metadata(&archive_path)?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;

    pb.inc(1);
    pb.finish_with_message(format!("{}: Complete ({:.2} MB)", book.id, size_mb));

    Ok(())
}
