//! Campaign Summary Service
//!
//! Manages cached AI-generated summaries of campaign story progress.
//! Summaries are built from session notes and module information.

use crate::connection::DbConnection;
use crate::dal::campaign::documents::DocumentRepository;
use crate::error::{DbError, Result};
use crate::services::{CampaignService, ModuleService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Cached campaign summary with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSummary {
    /// The generated summary text
    pub summary: String,
    /// ISO8601 timestamp when summary was generated
    pub generated_at: String,
    /// Hash of source materials used to generate summary
    pub source_hash: String,
    /// Campaign ID this summary is for
    pub campaign_id: i32,
}

/// Source materials used for generating summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySourceMaterial {
    /// Campaign name
    pub campaign_name: String,
    /// Module summaries (name + status)
    pub modules: Vec<ModuleSummaryInfo>,
    /// Session notes content (title + content)
    pub session_notes: Vec<SessionNoteInfo>,
}

/// Module information for summary generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummaryInfo {
    /// Module name
    pub name: String,
    /// Current status (planning, running, completed, etc.)
    pub status: String,
    /// Module number in sequence
    pub module_number: i32,
}

/// Session note information for summary generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNoteInfo {
    /// Document title
    pub title: String,
    /// Session number if known
    pub session_number: Option<i32>,
    /// Full content of the session notes
    pub content: String,
}

/// Service for managing campaign summaries
pub struct CampaignSummaryService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CampaignSummaryService<'a> {
    /// Create a new campaign summary service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Get the cache file path for a campaign
    fn get_cache_path(campaign_dir: &str) -> PathBuf {
        PathBuf::from(campaign_dir)
            .join(".mimir")
            .join("campaign_summary.json")
    }

    /// Read cached summary if it exists
    pub fn get_cached_summary(&self, campaign_dir: &str) -> Option<CampaignSummary> {
        let cache_path = Self::get_cache_path(campaign_dir);
        if cache_path.exists() {
            fs::read_to_string(&cache_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
        } else {
            None
        }
    }

    /// Save summary to cache
    pub fn save_summary(&self, campaign_dir: &str, summary: &CampaignSummary) -> Result<()> {
        let cache_path = Self::get_cache_path(campaign_dir);

        // Ensure .mimir directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DbError::InvalidData(format!("Failed to create .mimir directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(summary)
            .map_err(|e| DbError::InvalidData(format!("Failed to serialize summary: {}", e)))?;

        fs::write(&cache_path, json)
            .map_err(|e| DbError::InvalidData(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    /// Invalidate (delete) cached summary
    pub fn invalidate_cache(&self, campaign_dir: &str) -> Result<()> {
        let cache_path = Self::get_cache_path(campaign_dir);
        if cache_path.exists() {
            fs::remove_file(&cache_path).map_err(|e| {
                DbError::InvalidData(format!("Failed to delete cache file: {}", e))
            })?;
        }
        Ok(())
    }

    /// Calculate hash of source materials for staleness detection
    pub fn calculate_source_hash(source: &SummarySourceMaterial) -> String {
        let json = serde_json::to_string(source).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if cached summary is stale
    pub fn is_cache_stale(
        &mut self,
        campaign_id: i32,
        campaign_dir: &str,
    ) -> Result<(bool, SummarySourceMaterial)> {
        let source = self.gather_source_materials(campaign_id, campaign_dir)?;
        let current_hash = Self::calculate_source_hash(&source);

        if let Some(cached) = self.get_cached_summary(campaign_dir) {
            Ok((cached.source_hash != current_hash, source))
        } else {
            Ok((true, source)) // No cache = stale
        }
    }

    /// Gather all source materials for summary generation
    pub fn gather_source_materials(
        &mut self,
        campaign_id: i32,
        campaign_dir: &str,
    ) -> Result<SummarySourceMaterial> {
        // Get campaign name
        let campaign_name = {
            let mut service = CampaignService::new(self.conn);
            service
                .get_campaign(campaign_id)?
                .map(|c| c.name)
                .unwrap_or_else(|| "Unknown Campaign".to_string())
        };

        // Get modules
        let modules = {
            let mut service = ModuleService::new(self.conn);
            service
                .list_campaign_modules(campaign_id)
                .unwrap_or_default()
                .into_iter()
                .map(|m| ModuleSummaryInfo {
                    name: m.name,
                    status: m.status,
                    module_number: m.module_number,
                })
                .collect()
        };

        // Get session notes documents
        let session_notes = self.gather_session_notes(campaign_id, campaign_dir)?;

        Ok(SummarySourceMaterial {
            campaign_name,
            modules,
            session_notes,
        })
    }

    /// Gather session notes from documents
    fn gather_session_notes(
        &mut self,
        campaign_id: i32,
        campaign_dir: &str,
    ) -> Result<Vec<SessionNoteInfo>> {
        // Find documents that are session notes
        let documents = DocumentRepository::find_by_campaign(self.conn, campaign_id)?;

        let mut notes = Vec::new();
        for doc in documents {
            // Look for session_notes or session_plan type documents
            if doc.document_type == "session_notes"
                || doc.document_type == "session_plan"
                || doc.title.to_lowercase().contains("session")
                    && doc.title.to_lowercase().contains("note")
            {
                // Read file content
                let file_path = if doc.file_path.starts_with('/') {
                    PathBuf::from(&doc.file_path)
                } else {
                    PathBuf::from(campaign_dir).join(&doc.file_path)
                };

                if let Ok(content) = fs::read_to_string(&file_path) {
                    notes.push(SessionNoteInfo {
                        title: doc.title,
                        session_number: doc.session_id, // session_id maps to session number
                        content,
                    });
                }
            }
        }

        // Sort by session number
        notes.sort_by(|a, b| a.session_number.cmp(&b.session_number));

        Ok(notes)
    }

    /// Get summary, regenerating if stale (blocking)
    /// Returns the summary and whether it was regenerated
    ///
    /// Note: The actual LLM call must be provided by the caller since
    /// mimir-dm-core doesn't have LLM dependencies.
    pub fn get_summary_with_staleness_check(
        &mut self,
        campaign_id: i32,
        campaign_dir: &str,
    ) -> Result<(Option<CampaignSummary>, bool, SummarySourceMaterial)> {
        let (is_stale, source) = self.is_cache_stale(campaign_id, campaign_dir)?;

        if is_stale {
            Ok((None, true, source))
        } else {
            let cached = self.get_cached_summary(campaign_dir);
            Ok((cached, false, source))
        }
    }
}

/// Format source materials into a prompt for LLM summarization
pub fn format_source_for_llm(source: &SummarySourceMaterial) -> String {
    let mut prompt = format!(
        "Please summarize the story progress for the D&D campaign \"{}\".\n\n",
        source.campaign_name
    );

    // Add module info
    if !source.modules.is_empty() {
        prompt.push_str("## Adventure Modules:\n");
        for m in &source.modules {
            prompt.push_str(&format!(
                "- Module {}: {} (Status: {})\n",
                m.module_number, m.name, m.status
            ));
        }
        prompt.push('\n');
    }

    // Add session notes
    if !source.session_notes.is_empty() {
        prompt.push_str("## Session Notes:\n\n");
        for note in &source.session_notes {
            prompt.push_str(&format!("### {}\n", note.title));
            prompt.push_str(&note.content);
            prompt.push_str("\n\n");
        }
    }

    prompt.push_str(
        "\n---\n\
        Please provide a concise summary (2-4 paragraphs) covering:\n\
        1. The main storyline and plot progression\n\
        2. Key events and decisions\n\
        3. Current situation and immediate objectives\n\
        4. Important NPCs encountered\n\n\
        Write in past tense for events that happened, present tense for current situation.",
    );

    prompt
}
