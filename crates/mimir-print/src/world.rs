//! Custom Typst World implementation for Mimir
//!
//! A Typst "World" is the compiler's environment - it tells Typst where to find
//! source files, fonts, and images. Our implementation supports in-memory content
//! for the main document while resolving other resources from the filesystem.

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

use crate::embedded_templates::get_embedded_template;
use crate::error::{PrintError, Result};

/// Static font database loaded once
static FONT_DB: OnceLock<(LazyHash<FontBook>, Vec<Font>)> = OnceLock::new();

/// The fonts compiled into Mimir (MIMIR-T-0751): the templates' Inter and
/// JetBrains Mono, so a PDF has text whatever fonts the system has. A
/// system without a fontconfig setup (the server image had none) gives
/// `load_system_fonts` no fonts at all. Licenses: `fonts/*/OFL.txt`.
pub(crate) const BUNDLED_FONTS: [&[u8]; 6] = [
    include_bytes!("../fonts/inter/Inter-Regular.otf"),
    include_bytes!("../fonts/inter/Inter-Bold.otf"),
    include_bytes!("../fonts/inter/Inter-Italic.otf"),
    include_bytes!("../fonts/inter/Inter-BoldItalic.otf"),
    include_bytes!("../fonts/jetbrains-mono/JetBrainsMono-Regular.ttf"),
    include_bytes!("../fonts/jetbrains-mono/JetBrainsMono-Bold.ttf"),
];

/// Initialize the font database: the bundled fonts first, then the system
/// fonts as extras.
fn init_fonts() -> (LazyHash<FontBook>, Vec<Font>) {
    let mut book = FontBook::new();
    let mut fonts = Vec::new();

    for data in BUNDLED_FONTS {
        for font in Font::iter(Bytes::from_static(data)) {
            book.push(font.info().clone());
            fonts.push(font);
        }
    }

    let mut db = Database::new();
    db.load_system_fonts();

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
    /// Root directory for file resolution (templates, images, etc.)
    root_dir: PathBuf,
    /// The main file being compiled
    main_file: FileId,
    /// In-memory content for the main file (if provided)
    main_content: Option<String>,
    /// Cache of loaded source files
    sources: RwLock<HashMap<FileId, TypstSource>>,
    /// In-memory files (images, etc.) accessible via virtual paths
    virtual_files: RwLock<HashMap<String, Bytes>>,
    /// Library with standard functions
    library: LazyHash<Library>,
}

impl MimirTypstWorld {
    /// Create a world from in-memory Typst content
    ///
    /// Used by DocumentBuilder to compile generated Typst without writing to disk.
    /// The root_dir is still needed to resolve image/file references.
    pub fn from_content(content: String, root_dir: PathBuf) -> Self {
        let main_file = FileId::new(None, VirtualPath::new("main.typ"));
        Self {
            root_dir,
            main_file,
            main_content: Some(content),
            sources: RwLock::new(HashMap::new()),
            virtual_files: RwLock::new(HashMap::new()),
            library: LazyHash::new(Library::default()),
        }
    }

    /// Create a world from in-memory Typst content with pre-registered virtual files
    ///
    /// Virtual files can be referenced in Typst source using their registered paths.
    pub fn from_content_with_files(
        content: String,
        root_dir: PathBuf,
        files: HashMap<String, Vec<u8>>,
    ) -> Self {
        let main_file = FileId::new(None, VirtualPath::new("main.typ"));
        let virtual_files: HashMap<String, Bytes> = files
            .into_iter()
            .map(|(k, v)| (k, Bytes::from(v)))
            .collect();
        Self {
            root_dir,
            main_file,
            main_content: Some(content),
            sources: RwLock::new(HashMap::new()),
            virtual_files: RwLock::new(virtual_files),
            library: LazyHash::new(Library::default()),
        }
    }

    /// Register an in-memory file that can be accessed via a virtual path
    ///
    /// The path should be a simple name like "map_preview.png" which will be
    /// accessible as "/_virtual/map_preview.png" in Typst source.
    pub fn register_file(&self, name: &str, data: Vec<u8>) -> String {
        let virtual_path = format!("/_virtual/{}", name);
        if let Ok(mut files) = self.virtual_files.write() {
            files.insert(virtual_path.clone(), Bytes::from(data));
        }
        virtual_path
    }

    /// Create a world from a template file with JSON data injection
    ///
    /// The data is injected as `#let data = ...` at the top of the template.
    pub fn from_template(
        templates_root: PathBuf,
        template_path: &str,
        data: serde_json::Value,
    ) -> Result<Self> {
        let full_path = templates_root.join(template_path);
        if !full_path.exists() {
            return Err(PrintError::TemplateNotFound(template_path.to_string()));
        }

        // Read template and inject data
        let template_content = std::fs::read_to_string(&full_path)
            .map_err(|_| PrintError::TemplateReadError(template_path.to_string()))?;

        let data_json = serde_json::to_string(&data)?;
        let content = format!(
            "#let data = json.decode(\"{}\")\n\n{}",
            data_json.replace('\\', "\\\\").replace('"', "\\\""),
            template_content
        );

        let main_file = FileId::new(None, VirtualPath::new(template_path));
        Ok(Self {
            root_dir: templates_root,
            main_file,
            main_content: Some(content),
            sources: RwLock::new(HashMap::new()),
            virtual_files: RwLock::new(HashMap::new()),
            library: LazyHash::new(Library::default()),
        })
    }

    /// Resolve a file path to actual filesystem path
    fn resolve_path(&self, id: FileId) -> PathBuf {
        let vpath = id.vpath();
        let rooted = vpath.as_rooted_path();

        // Check if this looks like an absolute filesystem path
        let stripped = rooted.strip_prefix("/").unwrap_or(rooted);
        let as_absolute = PathBuf::from("/").join(stripped);
        if as_absolute.exists() {
            return as_absolute;
        }

        self.root_dir.join(stripped)
    }

    /// Read and cache a source file from disk
    fn read_source_from_disk(&self, id: FileId) -> FileResult<TypstSource> {
        // Check cache first
        if let Ok(sources) = self.sources.read() {
            if let Some(source) = sources.get(&id) {
                return Ok(source.clone());
            }
        }

        let path = self.resolve_path(id);
        let content = std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;

        let source = TypstSource::new(id, content);

        if let Ok(mut sources) = self.sources.write() {
            sources.insert(id, source.clone());
        }
        Ok(source)
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
        // Return in-memory content for main file
        if id == self.main_file {
            if let Some(ref content) = self.main_content {
                return Ok(TypstSource::new(id, content.clone()));
            }
        }

        // Check for embedded templates (/_shared/*.typ)
        let vpath = id.vpath().as_rooted_path();
        let vpath_str = vpath.to_string_lossy();
        if let Some(embedded_content) = get_embedded_template(&vpath_str) {
            tracing::debug!("Using embedded template: {}", vpath_str);
            return Ok(TypstSource::new(id, embedded_content.to_string()));
        }

        // Fall back to disk for other files
        self.read_source_from_disk(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        // Check for virtual files first
        let vpath = id.vpath().as_rooted_path();
        let vpath_str = vpath.to_string_lossy();

        tracing::debug!("Typst requesting file: {:?}", vpath_str);

        if let Ok(files) = self.virtual_files.read() {
            if let Some(data) = files.get(vpath_str.as_ref()) {
                tracing::debug!("Found in virtual files: {} bytes", data.len());
                return Ok(data.clone());
            }
        }

        // Fall back to filesystem
        let path = self.resolve_path(id);
        tracing::debug!("Resolved to filesystem path: {:?}", path);

        let data = std::fs::read(&path).map_err(|e| {
            tracing::warn!("Failed to read file {:?}: {}", path, e);
            FileError::from_io(e, &path)
        })?;
        tracing::debug!("Read {} bytes from filesystem", data.len());
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

    /// The bundled fonts parse and give the families the templates name
    /// (MIMIR-T-0751).
    #[test]
    fn bundled_fonts_give_the_template_families() {
        let mut families = Vec::new();
        for data in BUNDLED_FONTS {
            let fonts: Vec<Font> = Font::iter(Bytes::from_static(data)).collect();
            assert!(!fonts.is_empty(), "a bundled font does not parse");
            families.extend(fonts.iter().map(|f| f.info().family.clone()));
        }
        assert_eq!(
            families.iter().filter(|f| *f == "Inter").count(),
            4,
            "{families:?}"
        );
        assert_eq!(
            families.iter().filter(|f| *f == "JetBrains Mono").count(),
            2,
            "{families:?}"
        );
        let (book, _) = get_fonts();
        assert!(book.select_family("inter").next().is_some());
        assert!(book.select_family("jetbrains mono").next().is_some());
    }

    /// A PDF has its text in the bundled fonts, with no "unknown font
    /// family" warning — the server image once gave PDFs with no text.
    #[test]
    fn a_pdf_embeds_the_bundled_fonts() {
        let temp = TempDir::new().unwrap();
        let source = r#"#set text(font: "Inter")
#show raw: set text(font: "JetBrains Mono")
= The Inn Plain Sight

*Hot wings* and _a closed door_. `Dimension Door`
"#;
        let world = MimirTypstWorld::from_content(source.to_string(), temp.path().to_path_buf());
        let warned = typst::compile(&world);
        let warnings: Vec<String> = warned
            .warnings
            .iter()
            .map(|w| w.message.to_string())
            .collect();
        assert!(warnings.is_empty(), "{warnings:?}");
        let document = warned.output.expect("the document compiles");
        let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).expect("a PDF");
        let has = |needle: &[u8]| pdf.windows(needle.len()).any(|w| w == needle);
        for font in [
            &b"Inter-Regular"[..],
            b"Inter-Bold",
            b"Inter-Italic",
            b"JetBrainsMono-Regular",
        ] {
            assert!(
                has(font),
                "{} is not embedded",
                String::from_utf8_lossy(font)
            );
        }
    }

    #[test]
    fn test_from_content() {
        let temp = TempDir::new().unwrap();
        let world =
            MimirTypstWorld::from_content("= Hello World".to_string(), temp.path().to_path_buf());

        assert!(world.main_content.is_some());
        assert_eq!(world.main_content.unwrap(), "= Hello World");
    }

    #[test]
    fn test_from_template() {
        let temp = TempDir::new().unwrap();
        let template_path = temp.path().join("test.typ");
        fs::write(&template_path, "= Hello #data.name").unwrap();

        let world = MimirTypstWorld::from_template(
            temp.path().to_path_buf(),
            "test.typ",
            serde_json::json!({"name": "World"}),
        );

        assert!(world.is_ok());
        let world = world.unwrap();
        assert!(world.main_content.is_some());
        assert!(world.main_content.unwrap().contains("json.decode"));
    }

    #[test]
    fn test_template_not_found() {
        let temp = TempDir::new().unwrap();

        let world = MimirTypstWorld::from_template(
            temp.path().to_path_buf(),
            "nonexistent.typ",
            serde_json::json!({}),
        );

        assert!(matches!(world, Err(PrintError::TemplateNotFound(_))));
    }
}
