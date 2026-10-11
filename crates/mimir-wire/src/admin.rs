//! Settings and admin (MIMIR-T-0724): campaign archives, catalog imports
//! as background jobs, sources.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What a campaign archive holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArchiveCounts {
    pub modules: usize,
    pub documents: usize,
    pub characters: usize,
    pub maps: usize,
    pub tokens: usize,
    pub module_monsters: usize,
    pub module_npcs: usize,
    pub assets: usize,
    pub homebrew_items: usize,
    pub homebrew_monsters: usize,
    pub homebrew_spells: usize,
}

/// `POST /archives/preview`: an archive's manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivePreview {
    pub campaign_name: String,
    pub counts: ArchiveCounts,
    /// Catalog entries its documents name ("monster Goblin (MM)").
    pub catalog_references: Vec<String>,
    pub mimir_version: String,
    pub created_at: String,
}

/// `POST /archives/import`: the new campaign.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveImported {
    pub campaign_id: String,
    pub campaign_name: String,
    pub counts: ArchiveCounts,
}

/// A long piece of work (a catalog import); poll `GET /jobs/{id}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    /// "catalog" or "images".
    pub kind: String,
    /// "running", "done" or "failed".
    pub state: String,
    /// What it is doing, or what it did.
    pub message: String,
    #[serde(default)]
    pub result: Value,
}

/// `GET/PUT /campaigns/{id}/sources`, `/characters/{id}/sources`: the
/// source books it uses (empty: all).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Sources {
    pub codes: Vec<String>,
}
