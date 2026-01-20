//! Schema download utilities
//!
//! This module provides functions for downloading 5etools JSON schemas
//! from the upstream repository. It is only compiled when the
//! `schema-download` feature is enabled.
//!
//! # Usage
//!
//! ```rust,ignore
//! use mimir_dm_core::models::catalog::schema::download::download_all_schemas;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let output_dir = std::path::Path::new("./schema/vendored");
//!     download_all_schemas(output_dir).await?;
//!     Ok(())
//! }
//! ```

use std::path::Path;

use super::{paths, schema_url};

/// Download a single schema file
pub async fn download_schema(
    client: &reqwest::Client,
    schema_path: &str,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = schema_url(schema_path);
    let filename = Path::new(schema_path)
        .file_name()
        .ok_or("Invalid schema path")?;
    let output_path = output_dir.join(filename);

    tracing::info!("Downloading {} to {:?}", url, output_path);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download {}: HTTP {}",
            url,
            response.status()
        )
        .into());
    }

    let content = response.text().await?;

    // Validate it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content)?;

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, content)?;
    tracing::info!("Successfully downloaded {:?}", output_path);

    Ok(())
}

/// Download all key schemas
pub async fn download_all_schemas(
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("mimir-dm-schema-download/0.1")
        .build()?;

    let schemas = [
        paths::BESTIARY,
        paths::CLASS,
        paths::RACES,
        paths::SPELLS,
        paths::ITEMS,
        paths::BACKGROUNDS,
        paths::ENTRY,
        paths::UTIL,
    ];

    for schema_path in schemas {
        if let Err(e) = download_schema(&client, schema_path, output_dir).await {
            tracing::error!("Failed to download {}: {}", schema_path, e);
            // Continue with other schemas even if one fails
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_url_generation() {
        let url = schema_url(paths::BESTIARY);
        assert!(url.contains("5etools-utils"));
        assert!(url.contains("bestiary.json"));
    }
}
