//! Input source parsing and handling.

use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Source for 5etools data - either a local path or GitHub repository.
#[derive(Debug, Clone)]
pub enum InputSource {
    /// Local filesystem path to 5etools data.
    LocalPath(PathBuf),
    /// GitHub repository URL with optional reference (tag/branch/commit).
    GitHub {
        /// Repository URL.
        url: String,
        /// Optional git reference (tag, branch, or commit).
        reference: Option<String>,
    },
}

impl InputSource {
    /// Parse input string to determine if it's a local path or GitHub URL
    pub fn parse(input: &str) -> Result<Self> {
        // Check if it's a GitHub URL
        if input.starts_with("https://github.com/") || input.starts_with("git@github.com:") {
            Self::parse_github_url(input)
        } else if input.starts_with("http://") || input.starts_with("https://") {
            Err(anyhow!("Only GitHub URLs are supported. Got: {}", input))
        } else {
            // Treat as local path
            let path = PathBuf::from(input);
            if !path.exists() {
                return Err(anyhow!("Local path does not exist: {}", input));
            }
            Ok(InputSource::LocalPath(path))
        }
    }

    /// Parse GitHub URL with optional reference (tag/branch/commit)
    /// Format: https://github.com/org/repo.git[@ref]
    fn parse_github_url(url: &str) -> Result<Self> {
        let parts: Vec<&str> = url.splitn(2, '@').collect();
        let base_url = parts[0];
        let reference = parts.get(1).map(|s| s.to_string());

        // Validate it's a GitHub URL
        if !base_url.starts_with("https://github.com/") && !base_url.starts_with("git@github.com:")
        {
            return Err(anyhow!("Invalid GitHub URL: {}", url));
        }

        // Ensure .git suffix
        let url = if !base_url.ends_with(".git") {
            format!("{}.git", base_url)
        } else {
            base_url.to_string()
        };

        Ok(InputSource::GitHub { url, reference })
    }

    /// Detect if this is a 2014 ruleset based on URL or path
    pub fn is_2014_ruleset(&self) -> bool {
        match self {
            InputSource::LocalPath(path) => path.to_string_lossy().contains("2014"),
            InputSource::GitHub { url, .. } => url.contains("2014"),
        }
    }

    /// Get the corresponding image repository URL if this is a GitHub source
    pub fn get_image_repo_url(&self) -> Option<String> {
        match self {
            InputSource::GitHub { url, reference } => {
                let img_url = if url.contains("5etools-2014-src") {
                    url.replace("5etools-2014-src", "5etools-2014-img")
                } else if url.contains("5etools-src") {
                    url.replace("5etools-src", "5etools-img")
                } else {
                    return None;
                };

                // Add reference if present
                if let Some(ref_str) = reference {
                    Some(format!("{}@{}", img_url, ref_str))
                } else {
                    Some(img_url)
                }
            }
            InputSource::LocalPath(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_path() {
        let input = "/tmp";
        let source = InputSource::parse(input).unwrap();
        match source {
            InputSource::LocalPath(path) => assert_eq!(path, PathBuf::from("/tmp")),
            _ => panic!("Expected LocalPath"),
        }
    }

    #[test]
    fn test_parse_github_url() {
        let input = "https://github.com/5etools-mirror-3/5etools-src.git";
        let source = InputSource::parse(input).unwrap();
        match source {
            InputSource::GitHub { url, reference } => {
                assert_eq!(url, "https://github.com/5etools-mirror-3/5etools-src.git");
                assert!(reference.is_none());
            }
            _ => panic!("Expected GitHub"),
        }
    }

    #[test]
    fn test_parse_github_url_with_tag() {
        let input = "https://github.com/5etools-mirror-3/5etools-src.git@v1.210.46";
        let source = InputSource::parse(input).unwrap();
        match source {
            InputSource::GitHub { url, reference } => {
                assert_eq!(url, "https://github.com/5etools-mirror-3/5etools-src.git");
                assert_eq!(reference.unwrap(), "v1.210.46");
            }
            _ => panic!("Expected GitHub"),
        }
    }

    #[test]
    fn test_detect_2014_ruleset() {
        let source =
            InputSource::parse("https://github.com/5etools-mirror-3/5etools-2014-src.git").unwrap();
        assert!(source.is_2014_ruleset());

        let source =
            InputSource::parse("https://github.com/5etools-mirror-3/5etools-src.git").unwrap();
        assert!(!source.is_2014_ruleset());
    }

    #[test]
    fn test_get_image_repo_url() {
        let source =
            InputSource::parse("https://github.com/5etools-mirror-3/5etools-2014-src.git").unwrap();
        assert_eq!(
            source.get_image_repo_url(),
            Some("https://github.com/5etools-mirror-3/5etools-2014-img.git".to_string())
        );

        let source =
            InputSource::parse("https://github.com/5etools-mirror-3/5etools-src.git@v1.210.46")
                .unwrap();
        assert_eq!(
            source.get_image_repo_url(),
            Some("https://github.com/5etools-mirror-3/5etools-img.git@v1.210.46".to_string())
        );
    }
}
