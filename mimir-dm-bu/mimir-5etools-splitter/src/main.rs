use anyhow::Result;
use clap::{Parser, Subcommand};
use mimir_5etools_splitter::{extract_srd, split_repository, InputSource};
use std::path::PathBuf;

/// 5etools repository splitter and SRD extractor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Split 5etools repository into book-specific archives
    Split {
        /// Input source: local directory path or GitHub URL[@tag]
        ///
        /// Examples:
        ///   /path/to/5etools-src
        ///   https://github.com/5etools-mirror-3/5etools-2014-src.git
        ///   https://github.com/5etools-mirror-3/5etools-2014-src.git@v1.210.46
        #[arg(value_name = "INPUT")]
        input: String,

        /// Output directory for archive files
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
    },
    /// Extract SRD (System Reference Document) content
    ExtractSrd {
        /// Input source: local directory path or GitHub URL[@tag]
        #[arg(value_name = "INPUT")]
        input: String,

        /// Output directory for SRD archive
        #[arg(value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    match args.command {
        Commands::Split { input, output_dir } => {
            // Parse input source
            let input_source = InputSource::parse(&input)?;

            // Display what we're doing
            match &input_source {
                InputSource::LocalPath(path) => {
                    println!("📁 Processing local repository: {:?}", path);
                }
                InputSource::GitHub { url, reference } => {
                    if let Some(ref_str) = reference {
                        println!("🌐 Cloning from GitHub: {} @ {}", url, ref_str);
                    } else {
                        println!("🌐 Cloning from GitHub: {} @ latest", url);
                    }
                }
            }

            println!("📦 Output directory: {:?}", output_dir);

            // Process the repository
            println!("\n🚀 Starting processing...\n");
            let results = split_repository(input_source, &output_dir).await?;

            // Display results
            println!("\n✨ Processing complete!");
            println!("📊 Total books processed: {}", results.total_processed);
            println!("✅ Successful: {}", results.successful.len());

            if !results.successful.is_empty() {
                println!("\nSuccessfully created archives for:");
                for book_id in &results.successful {
                    println!("  • {}.tar.gz", book_id.to_lowercase());
                }
            }

            if !results.failed.is_empty() {
                println!("\n❌ Failed: {}", results.failed.len());
                println!("\nFailed to process:");
                for (book_id, error) in &results.failed {
                    println!("  • {}: {}", book_id, error);
                }
            }
        }

        Commands::ExtractSrd { input, output_dir } => {
            // Parse input source
            let input_source = InputSource::parse(&input)?;

            // Display what we're doing
            match &input_source {
                InputSource::LocalPath(path) => {
                    println!("📜 Extracting SRD from local repository: {:?}", path);
                }
                InputSource::GitHub { url, reference } => {
                    if let Some(ref_str) = reference {
                        println!("📜 Extracting SRD from GitHub: {} @ {}", url, ref_str);
                    } else {
                        println!("📜 Extracting SRD from GitHub: {} @ latest", url);
                    }
                }
            }

            println!("📦 Output directory: {:?}", output_dir);

            // Extract SRD content
            println!("\n⚡ Starting SRD extraction...\n");
            let results = extract_srd(input_source, &output_dir).await?;

            // Display results
            println!("\n✨ SRD extraction complete!");
            println!("📊 Total SRD items: {}", results.total_items);
            println!("📁 Archive created: {}", results.archive_path);

            println!("\n📋 Content Summary:");
            let mut sorted_content: Vec<_> = results.content_summary.iter().collect();
            sorted_content.sort_by_key(|(_, count)| *count);
            sorted_content.reverse();

            for (content_type, count) in sorted_content {
                println!("  • {}: {} items", content_type, count);
            }

            println!("\nℹ️  This archive contains only SRD (Open Game Content) and can be freely distributed.");
        }
    }

    Ok(())
}
