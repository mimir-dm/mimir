//! Custom Typst World implementation for Mimir
//!
//! The World trait is how Typst resolves files, fonts, and other resources.
//! This implementation provides:
//! - Template file resolution from our templates directory
//! - System font loading
//! - Data injection via JSON

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use chrono::{Datelike, Timelike};
use fontdb::{Database, Source};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source as TypstSource, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;

use crate::error::{PrintError, Result};

/// Static font database loaded once
static FONT_DB: OnceLock<(LazyHash<FontBook>, Vec<Font>)> = OnceLock::new();

/// Initialize the font database from system fonts
fn init_fonts() -> (LazyHash<FontBook>, Vec<Font>) {
    let mut db = Database::new();
    db.load_system_fonts();

    let mut book = FontBook::new();
    let mut fonts = Vec::new();

    for face in db.faces() {
        let path = match &face.source {
            Source::File(path) | Source::SharedFile(path, _) => path,
            Source::Binary(_) => continue,
        };

        let data = match std::fs::read(path) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let bytes = Bytes::from(data);
        for (i, font) in Font::iter(bytes).enumerate() {
            if i == face.index as usize {
                book.push(font.info().clone());
                fonts.push(font);
                break;
            }
        }
    }

    (LazyHash::new(book), fonts)
}

/// Get or initialize the font database
fn get_fonts() -> &'static (LazyHash<FontBook>, Vec<Font>) {
    FONT_DB.get_or_init(init_fonts)
}

/// Mimir's custom Typst World implementation
pub struct MimirTypstWorld {
    /// Root directory for templates
    templates_root: PathBuf,
    /// The main template file being compiled
    main_file: FileId,
    /// Cache of loaded source files
    sources: RwLock<HashMap<FileId, TypstSource>>,
    /// Library with standard functions
    library: LazyHash<Library>,
    /// Data to inject into templates (as JSON string)
    data_json: String,
}

impl MimirTypstWorld {
    /// Create a new MimirTypstWorld
    ///
    /// # Arguments
    /// * `templates_root` - Root directory containing templates
    /// * `template_path` - Path to the main template file (relative to templates_root)
    /// * `data` - JSON data to inject into the template
    pub fn new(
        templates_root: PathBuf,
        template_path: &str,
        data: serde_json::Value,
    ) -> Result<Self> {
        let full_path = templates_root.join(template_path);
        if !full_path.exists() {
            return Err(PrintError::TemplateNotFound(template_path.to_string()));
        }

        let main_file = FileId::new(None, VirtualPath::new(template_path));
        let data_json = serde_json::to_string(&data)?;

        Ok(Self {
            templates_root,
            main_file,
            sources: RwLock::new(HashMap::new()),
            library: LazyHash::new(Library::default()),
            data_json,
        })
    }

    /// Resolve a file path to actual filesystem path
    fn resolve_path(&self, id: FileId) -> PathBuf {
        let vpath = id.vpath();
        let rooted = vpath.as_rooted_path();

        // Check if this looks like an absolute filesystem path (e.g., /var/folders/...)
        // If so, and it exists, return it directly without joining to templates_root
        let stripped = rooted.strip_prefix("/").unwrap_or(rooted);
        let as_absolute = PathBuf::from("/").join(stripped);
        if as_absolute.exists() {
            return as_absolute;
        }

        // Otherwise, resolve relative to templates root
        self.templates_root.join(stripped)
    }

    /// Read and cache a source file
    fn read_source(&self, id: FileId) -> FileResult<TypstSource> {
        // Check if already cached (read lock)
        if let Ok(sources) = self.sources.read() {
            if let Some(source) = sources.get(&id) {
                return Ok(source.clone());
            }
        }

        let path = self.resolve_path(id);
        let content =
            std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;

        // If this is the main file, inject data
        let content = if id == self.main_file {
            self.inject_data(&content)
        } else {
            content
        };

        let source = TypstSource::new(id, content);

        // Cache the source (write lock)
        if let Ok(mut sources) = self.sources.write() {
            sources.insert(id, source.clone());
        }
        Ok(source)
    }

    /// Inject JSON data into template by prepending a data variable
    fn inject_data(&self, content: &str) -> String {
        format!(
            "#let data = json.decode(\"{}\")\n\n{}",
            self.data_json.replace('\\', "\\\\").replace('"', "\\\""),
            content
        )
    }
}

impl typst::World for MimirTypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        let (book, _) = get_fonts();
        book
    }

    fn main(&self) -> FileId {
        self.main_file
    }

    fn source(&self, id: FileId) -> FileResult<TypstSource> {
        self.read_source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve_path(id);
        let data = std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Bytes::from(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        let (_, fonts) = get_fonts();
        fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Local::now();
        let offset_duration = chrono::Duration::hours(offset.unwrap_or(0));
        let naive = now.naive_utc() + offset_duration;
        Datetime::from_ymd_hms(
            naive.date().year(),
            naive.date().month() as u8,
            naive.date().day() as u8,
            naive.time().hour() as u8,
            naive.time().minute() as u8,
            naive.time().second() as u8,
        )
    }

    fn packages(&self) -> &[(PackageSpec, Option<ecow::EcoString>)] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_world_creation() {
        let temp = TempDir::new().unwrap();
        let template_path = temp.path().join("test.typ");
        fs::write(&template_path, "Hello").unwrap();

        let world = MimirTypstWorld::new(
            temp.path().to_path_buf(),
            "test.typ",
            serde_json::json!({}),
        );

        assert!(world.is_ok());
    }

    #[test]
    fn test_template_not_found() {
        let temp = TempDir::new().unwrap();

        let world = MimirTypstWorld::new(
            temp.path().to_path_buf(),
            "nonexistent.typ",
            serde_json::json!({}),
        );

        assert!(matches!(world, Err(PrintError::TemplateNotFound(_))));
    }
}
