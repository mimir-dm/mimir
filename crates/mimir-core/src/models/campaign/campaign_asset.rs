//! Campaign Asset Model
//!
//! User-uploaded images for campaigns or modules (maps, props, puzzles, etc.).

use crate::schema::campaign_assets;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Allowed MIME types for campaign assets.
pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/svg+xml",
    "image/gif",
    "application/octet-stream", // For .dd2vtt UVTT files
];

/// A campaign asset - user-uploaded file (image, UVTT map, etc.).
/// Can belong to either a campaign or a module (exactly one).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = campaign_assets)]
pub struct CampaignAsset {
    /// Unique asset ID (UUID)
    pub id: String,
    /// Campaign this asset belongs to (if campaign-level)
    pub campaign_id: Option<String>,
    /// Module this asset belongs to (if module-level)
    pub module_id: Option<String>,
    /// Original filename
    pub filename: String,
    /// Optional user description
    pub description: Option<String>,
    /// MIME type (e.g., "image/png")
    pub mime_type: String,
    /// Relative path in app data directory
    pub blob_path: String,
    /// File size in bytes
    pub file_size: Option<i32>,
    /// ISO8601 timestamp of upload
    pub uploaded_at: String,
}

impl CampaignAsset {
    /// Check if the asset is an image.
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Check if the asset is a UVTT map file.
    pub fn is_uvtt(&self) -> bool {
        self.filename.ends_with(".dd2vtt") || self.filename.ends_with(".uvtt")
    }

    /// Get the file extension from the filename.
    pub fn extension(&self) -> Option<&str> {
        self.filename.rsplit('.').next()
    }

    /// Check if this is a campaign-level asset.
    pub fn is_campaign_asset(&self) -> bool {
        self.campaign_id.is_some()
    }

    /// Check if this is a module-level asset.
    pub fn is_module_asset(&self) -> bool {
        self.module_id.is_some()
    }
}

/// Data for inserting a new campaign asset.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaign_assets)]
pub struct NewCampaignAsset<'a> {
    pub id: &'a str,
    pub campaign_id: Option<&'a str>,
    pub module_id: Option<&'a str>,
    pub filename: &'a str,
    pub description: Option<&'a str>,
    pub mime_type: &'a str,
    pub blob_path: &'a str,
    pub file_size: Option<i32>,
}

impl<'a> NewCampaignAsset<'a> {
    /// Create a new campaign-level asset.
    pub fn for_campaign(
        id: &'a str,
        campaign_id: &'a str,
        filename: &'a str,
        mime_type: &'a str,
        blob_path: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id: Some(campaign_id),
            module_id: None,
            filename,
            description: None,
            mime_type,
            blob_path,
            file_size: None,
        }
    }

    /// Create a new module-level asset.
    pub fn for_module(
        id: &'a str,
        module_id: &'a str,
        filename: &'a str,
        mime_type: &'a str,
        blob_path: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id: None,
            module_id: Some(module_id),
            filename,
            description: None,
            mime_type,
            blob_path,
            file_size: None,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the file size.
    pub fn with_file_size(mut self, size: i32) -> Self {
        self.file_size = Some(size);
        self
    }
}

/// Validate that a MIME type is allowed.
pub fn is_allowed_mime_type(mime_type: &str) -> bool {
    ALLOWED_MIME_TYPES.contains(&mime_type)
}

/// Get file extension for a MIME type.
pub fn extension_for_mime_type(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/svg+xml" => Some("svg"),
        "image/gif" => Some("gif"),
        "application/octet-stream" => Some("bin"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_campaign_asset_for_campaign() {
        let asset = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "treasure_map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        assert_eq!(asset.id, "asset-1");
        assert_eq!(asset.campaign_id, Some("camp-1"));
        assert!(asset.module_id.is_none());
        assert_eq!(asset.filename, "treasure_map.png");
        assert_eq!(asset.mime_type, "image/png");
        assert!(asset.file_size.is_none());
    }

    #[test]
    fn test_new_campaign_asset_for_module() {
        let asset = NewCampaignAsset::for_module(
            "asset-1",
            "mod-1",
            "dungeon_map.dd2vtt",
            "application/octet-stream",
            "assets/mod-1/asset-1.dd2vtt",
        );
        assert_eq!(asset.id, "asset-1");
        assert!(asset.campaign_id.is_none());
        assert_eq!(asset.module_id, Some("mod-1"));
        assert_eq!(asset.filename, "dungeon_map.dd2vtt");
    }

    #[test]
    fn test_new_campaign_asset_with_size() {
        let asset = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "treasure_map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        )
        .with_file_size(1024);
        assert_eq!(asset.file_size, Some(1024));
    }

    #[test]
    fn test_is_allowed_mime_type() {
        assert!(is_allowed_mime_type("image/png"));
        assert!(is_allowed_mime_type("image/jpeg"));
        assert!(is_allowed_mime_type("image/svg+xml"));
        assert!(is_allowed_mime_type("application/octet-stream"));
        assert!(!is_allowed_mime_type("text/html"));
        assert!(!is_allowed_mime_type("application/pdf"));
    }

    #[test]
    fn test_extension_for_mime_type() {
        assert_eq!(extension_for_mime_type("image/png"), Some("png"));
        assert_eq!(extension_for_mime_type("image/jpeg"), Some("jpg"));
        assert_eq!(extension_for_mime_type("image/svg+xml"), Some("svg"));
        assert_eq!(extension_for_mime_type("text/html"), None);
    }
}
