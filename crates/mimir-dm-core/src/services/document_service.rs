//! Document management service.
//!
//! Provides business logic for managing campaign documents. Documents include
//! campaign-level materials, module planning docs, session notes, and player
//! handouts. Handles template rendering, file system operations, and database
//! record management.

use crate::connection::DbConnection;
use crate::dal::campaign::{
    campaigns::CampaignRepository, documents::DocumentRepository,
    template_documents::TemplateRepository,
};
use crate::error::Result;
use crate::models::campaign::documents::{Document, FileType, NewDocument, UpdateDocument};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;
use std::path::PathBuf;

/// Service for managing documents
pub struct DocumentService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> DocumentService<'a> {
    /// Create a new document service.
    ///
    /// # Arguments
    /// * `conn` - Mutable reference to the database connection
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Get all documents for a campaign.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - All documents associated with the campaign
    pub fn get_campaign_documents(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_by_campaign(self.conn, campaign_id)
    }

    /// Get documents by level with filtering logic.
    ///
    /// Filters documents by their scope in the campaign hierarchy.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `level` - Document scope: "campaign", "module", "session", or "handout"
    /// * `module_id` - Required for "module" level filtering
    /// * `session_id` - Required for "session" level filtering
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - Documents matching the level and filters
    /// * `Err(DbError::InvalidData)` - If level is invalid
    pub fn get_documents_by_level(
        &mut self,
        campaign_id: i32,
        level: &str,
        module_id: Option<i32>,
        session_id: Option<i32>,
    ) -> Result<Vec<Document>> {
        match level {
            "campaign" => {
                // Get campaign-level documents (no module or session id)
                DocumentRepository::find_by_campaign(self.conn, campaign_id).map(|docs| {
                    docs.into_iter()
                        .filter(|d| {
                            d.module_id.is_none()
                                && d.session_id.is_none()
                                && d.document_type != "handout"
                        })
                        .collect()
                })
            }
            "module" => {
                if let Some(mid) = module_id {
                    DocumentRepository::find_by_module(self.conn, mid)
                } else {
                    Ok(vec![])
                }
            }
            "session" => {
                if let Some(sid) = session_id {
                    DocumentRepository::find_by_session(self.conn, sid)
                } else {
                    Ok(vec![])
                }
            }
            "handout" => DocumentRepository::find_handouts_by_campaign(self.conn, campaign_id),
            _ => Err(crate::error::DbError::InvalidData(format!(
                "Invalid document level: {}",
                level
            ))),
        }
    }

    /// Create a new document.
    ///
    /// Creates a database record for an existing document file. Does not
    /// create the file on disk.
    ///
    /// # Arguments
    /// * `new_document` - Document creation data
    ///
    /// # Returns
    /// * `Ok(Document)` - The created document record
    pub fn create_document(&mut self, new_document: NewDocument) -> Result<Document> {
        DocumentRepository::create(self.conn, new_document)
    }

    /// Update a document.
    ///
    /// # Arguments
    /// * `document_id` - Database ID of the document
    /// * `update` - Fields to update (None fields are left unchanged)
    ///
    /// # Returns
    /// * `Ok(Document)` - The updated document record
    pub fn update_document(
        &mut self,
        document_id: i32,
        update: UpdateDocument,
    ) -> Result<Document> {
        DocumentRepository::update(self.conn, document_id, update)
    }

    /// Mark a document as completed.
    ///
    /// Sets the completed_at timestamp, which affects stage progression checks.
    ///
    /// # Arguments
    /// * `document_id` - Database ID of the document
    ///
    /// # Returns
    /// * `Ok(Document)` - The updated document with completed_at set
    pub fn complete_document(&mut self, document_id: i32) -> Result<Document> {
        DocumentRepository::mark_completed(self.conn, document_id)
    }

    /// Delete a document.
    ///
    /// Removes the database record. Does not delete the file on disk.
    ///
    /// # Arguments
    /// * `document_id` - Database ID of the document
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_document(&mut self, document_id: i32) -> Result<()> {
        DocumentRepository::delete(self.conn, document_id).map(|_| ())
    }

    /// Get incomplete documents for a campaign.
    ///
    /// Returns documents that have not been marked as completed.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - Documents where completed_at is NULL
    pub fn get_incomplete_documents(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_incomplete_by_campaign(self.conn, campaign_id)
    }

    /// Get completed documents for a campaign.
    ///
    /// Returns documents that have been marked as completed.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - Documents where completed_at is set
    pub fn get_completed_documents(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_completed_by_campaign(self.conn, campaign_id)
    }

    /// Create a document from a template.
    ///
    /// Retrieves the template, renders it with default values, writes the file
    /// to the campaign directory, and creates the database record.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `template_id` - ID of the template to use
    ///
    /// # Returns
    /// * `Ok(Document)` - The created document record
    /// * `Err(DbError::NotFound)` - If campaign or template not found
    /// * `Err(DbError::InvalidData)` - If document already exists
    pub fn create_document_from_template(
        &mut self,
        campaign_id: i32,
        template_id: &str,
    ) -> Result<Document> {
        // Get the campaign
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo.find_by_id(campaign_id)?.ok_or_else(|| {
            crate::error::DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            }
        })?;

        // Check if document already exists
        let existing = DocumentRepository::find_by_campaign(self.conn, campaign_id)?;
        if existing.iter().any(|d| d.template_id == template_id) {
            return Err(crate::error::DbError::InvalidData(
                "Document already exists".into(),
            ));
        }

        // Get the template
        let template = TemplateRepository::get_latest(self.conn, template_id)?;

        // Create the document file
        let file_name = format!("{}.md", template_id);
        let file_path = PathBuf::from(&campaign.directory_path).join(&file_name);

        // Process template using the create_context method
        let context = template.create_context();
        let mut tera = tera::Tera::default();
        tera.add_raw_template(&template.document_id, &template.document_content)
            .map_err(|e| {
                crate::error::DbError::InvalidData(format!("Failed to add template: {}", e))
            })?;

        let template_content = tera.render(&template.document_id, &context).map_err(|e| {
            crate::error::DbError::InvalidData(format!("Failed to render template: {}", e))
        })?;

        // Generate title from template_id (e.g., "campaign_pitch" -> "Campaign Pitch")
        let title = template_id
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Document type uses underscores (e.g., "campaign_pitch")
        let document_type = template_id.replace('-', "_");

        // Add YAML frontmatter with title and type
        let content_with_frontmatter = format!(
            "---\ntitle: \"{}\"\ntype: {}\n---\n\n{}",
            title, document_type, template_content
        );

        // Write file to disk with frontmatter
        fs::write(&file_path, content_with_frontmatter)?;

        let new_doc = NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template_id.to_string(),
            document_type,
            title,
            file_path: file_path.to_string_lossy().to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        };

        DocumentRepository::create(self.conn, new_doc)
    }

    /// Read a document file from disk.
    ///
    /// # Arguments
    /// * `file_path` - Absolute path to the document file
    ///
    /// # Returns
    /// * `Ok(String)` - The file contents
    /// * `Err(DbError::NotFound)` - If the file does not exist
    /// * `Err(DbError::Io)` - If reading fails
    pub fn read_document_file(&self, file_path: &str) -> Result<String> {
        let path = PathBuf::from(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(crate::error::DbError::NotFound {
                entity_type: "Document file".to_string(),
                id: file_path.to_string(),
            });
        }

        // Read the markdown file directly
        Ok(fs::read_to_string(&path)?)
    }

    /// Save a document file to disk.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    /// * `file_path` - Absolute path to the document file
    /// * `content` - The content to write
    ///
    /// # Returns
    /// * `Ok(())` - If writing succeeds
    /// * `Err(DbError::Io)` - If writing fails
    pub fn save_document_file(&self, file_path: &str, content: &str) -> Result<()> {
        let path = PathBuf::from(file_path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Write the file
        fs::write(&path, content)?;
        Ok(())
    }

    /// Create a new user document (markdown).
    ///
    /// Creates a blank markdown file with frontmatter and registers it in the database.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `module_id` - Optional module ID for module-level documents
    /// * `title` - Title for the document
    /// * `content` - Optional initial content (defaults to empty)
    ///
    /// # Returns
    /// * `Ok(Document)` - The created document record
    pub fn create_user_document(
        &mut self,
        campaign_id: i32,
        module_id: Option<i32>,
        title: &str,
        content: Option<&str>,
    ) -> Result<Document> {
        // Get the campaign directory
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo.find_by_id(campaign_id)?.ok_or_else(|| {
            crate::error::DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            }
        })?;

        // Generate filename from title
        let filename = Self::sanitize_filename(title) + ".md";

        // Determine target directory
        let user_docs_dir = Self::get_user_documents_dir(&campaign.directory_path, module_id);
        fs::create_dir_all(&user_docs_dir)?;

        let file_path = user_docs_dir.join(&filename);

        // Check for collision
        if file_path.exists() {
            return Err(crate::error::DbError::InvalidData(
                format!("A document with this name already exists: {}", filename),
            ));
        }

        // Create content with frontmatter
        let initial_content = content.unwrap_or("");
        let content_with_frontmatter = format!(
            "---\ntitle: \"{}\"\ntype: user_document\n---\n\n{}",
            title, initial_content
        );

        // Write file
        fs::write(&file_path, &content_with_frontmatter)?;

        // Create database record
        let new_doc = NewDocument {
            campaign_id,
            module_id,
            session_id: None,
            template_id: "user_document".to_string(),
            document_type: "user_document".to_string(),
            title: title.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_type: "markdown".to_string(),
            is_user_created: true,
        };

        DocumentRepository::create(self.conn, new_doc)
    }

    /// Upload a document file (markdown or image).
    ///
    /// Saves the file to the user-documents directory and creates a database record.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `module_id` - Optional module ID for module-level documents
    /// * `filename` - Original filename with extension
    /// * `data` - File contents (for markdown: string content, for images: base64-encoded)
    /// * `is_base64` - Whether data is base64-encoded (true for images)
    ///
    /// # Returns
    /// * `Ok(Document)` - The created document record
    pub fn upload_document(
        &mut self,
        campaign_id: i32,
        module_id: Option<i32>,
        filename: &str,
        data: &str,
        is_base64: bool,
    ) -> Result<Document> {
        // Determine file type from extension
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("md");

        let file_type = FileType::from_extension(extension).ok_or_else(|| {
            crate::error::DbError::InvalidData(format!(
                "Unsupported file type: {}. Supported types: md, png, jpg, jpeg, webp, gif, svg",
                extension
            ))
        })?;

        // Get the campaign directory
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo.find_by_id(campaign_id)?.ok_or_else(|| {
            crate::error::DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            }
        })?;

        // Sanitize filename
        let safe_filename = Self::sanitize_filename(
            std::path::Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document"),
        ) + "." + extension;

        // Determine target directory
        let user_docs_dir = Self::get_user_documents_dir(&campaign.directory_path, module_id);
        fs::create_dir_all(&user_docs_dir)?;

        let file_path = user_docs_dir.join(&safe_filename);

        // Check for collision - generate unique name if needed
        let final_path = Self::get_unique_path(&file_path);

        // Write file
        if is_base64 {
            let bytes = BASE64.decode(data).map_err(|e| {
                crate::error::DbError::InvalidData(format!("Invalid base64 data: {}", e))
            })?;
            fs::write(&final_path, bytes)?;
        } else {
            fs::write(&final_path, data)?;
        }

        // Extract title from filename
        let title = final_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .replace('_', " ")
            .replace('-', " ");

        // Create database record
        let new_doc = NewDocument {
            campaign_id,
            module_id,
            session_id: None,
            template_id: "user_document".to_string(),
            document_type: "user_document".to_string(),
            title,
            file_path: final_path.to_string_lossy().to_string(),
            file_type: file_type.as_str().to_string(),
            is_user_created: true,
        };

        DocumentRepository::create(self.conn, new_doc)
    }

    /// Read an image document as a base64 data URL.
    ///
    /// # Arguments
    /// * `file_path` - Absolute path to the image file
    ///
    /// # Returns
    /// * `Ok(String)` - Data URL (e.g., "data:image/png;base64,...")
    pub fn read_image_document(&self, file_path: &str) -> Result<String> {
        let path = PathBuf::from(file_path);

        if !path.exists() {
            return Err(crate::error::DbError::NotFound {
                entity_type: "Image file".to_string(),
                id: file_path.to_string(),
            });
        }

        // Determine MIME type from extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let file_type = FileType::from_extension(extension).ok_or_else(|| {
            crate::error::DbError::InvalidData(format!("Unsupported file type: {}", extension))
        })?;

        // Read and encode
        let bytes = fs::read(&path)?;
        let base64_data = BASE64.encode(&bytes);

        Ok(format!("data:{};base64,{}", file_type.mime_type(), base64_data))
    }

    /// Get all user-created documents for a campaign or module.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `module_id` - Optional module ID to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - User-created documents
    pub fn get_user_documents(
        &mut self,
        campaign_id: i32,
        module_id: Option<i32>,
    ) -> Result<Vec<Document>> {
        let all_docs = DocumentRepository::find_by_campaign(self.conn, campaign_id)?;

        Ok(all_docs
            .into_iter()
            .filter(|d| {
                d.is_user_created
                    && (module_id.is_none() || d.module_id == module_id)
            })
            .collect())
    }

    /// Get the user-documents directory path for a campaign or module.
    fn get_user_documents_dir(campaign_dir: &str, module_id: Option<i32>) -> PathBuf {
        let base = PathBuf::from(campaign_dir);
        match module_id {
            Some(mid) => base.join(format!("modules/module_{:02}", mid)).join("user-documents"),
            None => base.join("user-documents"),
        }
    }

    /// Sanitize a filename by removing/replacing invalid characters.
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ if c.is_ascii_control() => '_',
                _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Get a unique file path by appending a number if the file exists.
    fn get_unique_path(path: &PathBuf) -> PathBuf {
        if !path.exists() {
            return path.clone();
        }

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let parent = path.parent().unwrap_or(path);

        for i in 1..1000 {
            let new_name = if extension.is_empty() {
                format!("{}_{}", stem, i)
            } else {
                format!("{}_{}.{}", stem, i, extension)
            };
            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
        }

        // Fallback - use timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, timestamp)
        } else {
            format!("{}_{}.{}", stem, timestamp, extension)
        };
        parent.join(new_name)
    }
}
