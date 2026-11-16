//! Files in and out of the server for the MCP tools (MIMIR-A-0012,
//! MIMIR-T-0747).
//!
//! The tools run on the server, so a path in a tool's arguments names a
//! file in the server's folders, not a file on the caller's machine. The
//! bytes go over plain HTTP instead: the caller uploads a file to
//! `POST /api/v1/uploads` and gives a tool the `upload_id`; a tool that
//! makes a file keeps it here and gives a `download_url` for
//! `GET /api/v1/downloads/{id}`. MCP could carry a file only as base64
//! text in a tool call, which the model would have to write out.
//!
//! Layout: `<app_dir>/tmp/mcp-uploads/<id>/<file>` and
//! `<app_dir>/tmp/mcp-downloads/<id>/<file>`. An entry older than [`TTL`]
//! goes at the next save. A download goes when it is read.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde_json::{json, Value};

use crate::{McpContext, McpError};

/// How long an upload or a download is kept.
pub const TTL: Duration = Duration::from_secs(60 * 60);
/// The upload endpoint, under the server's address.
pub const UPLOADS_PATH: &str = "/api/v1/uploads";
/// The download endpoint, under the server's address.
pub const DOWNLOADS_PATH: &str = "/api/v1/downloads";

/// Uploads (files in) or downloads (files out).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Upload,
    Download,
}

impl Kind {
    fn folder(self) -> &'static str {
        match self {
            Kind::Upload => "mcp-uploads",
            Kind::Download => "mcp-downloads",
        }
    }
}

/// A kept file.
#[derive(Debug, Clone)]
pub struct Stored {
    pub id: String,
    pub filename: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

/// The folder of the entries of one kind.
pub fn root(app_dir: &Path, kind: Kind) -> PathBuf {
    app_dir.join("tmp").join(kind.folder())
}

/// The last part of a file name, with only safe characters; `default`
/// when nothing is left.
pub fn clean_name(name: &str, default: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let safe: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-+".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe = safe.trim_start_matches('.');
    if safe.is_empty() {
        default.to_string()
    } else {
        safe.to_string()
    }
}

/// A new, empty entry: its id and its folder. Removes the old entries of
/// the kind first.
pub fn new_entry(app_dir: &Path, kind: Kind) -> std::io::Result<(String, PathBuf)> {
    sweep(app_dir, kind);
    let id = uuid::Uuid::new_v4().to_string();
    let dir = root(app_dir, kind).join(&id);
    std::fs::create_dir_all(&dir)?;
    Ok((id, dir))
}

/// Keep `bytes` as a new entry.
pub fn save(app_dir: &Path, kind: Kind, filename: &str, bytes: &[u8]) -> std::io::Result<Stored> {
    let (id, dir) = new_entry(app_dir, kind)?;
    let filename = clean_name(filename, "file");
    let path = dir.join(&filename);
    std::fs::write(&path, bytes)?;
    Ok(Stored {
        id,
        filename,
        path,
        size_bytes: bytes.len() as u64,
    })
}

/// The file of an entry; `None` when the id is not an entry, or the entry
/// is too old.
pub fn find(app_dir: &Path, kind: Kind, id: &str) -> Option<Stored> {
    // An id is a UUID: never a path.
    uuid::Uuid::parse_str(id).ok()?;
    let dir = root(app_dir, kind).join(id);
    if is_old(&dir) {
        let _ = std::fs::remove_dir_all(&dir);
        return None;
    }
    let file = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_file())?;
    Some(Stored {
        id: id.to_string(),
        filename: file.file_name()?.to_string_lossy().into_owned(),
        size_bytes: std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0),
        path: file,
    })
}

/// Take a download for one read: the entry moves aside, so a second
/// `take` of the same id finds nothing. The caller removes the returned
/// folder when the read ends.
pub fn take(app_dir: &Path, kind: Kind, id: &str) -> Option<(Stored, PathBuf)> {
    let found = find(app_dir, kind, id)?;
    let dir = root(app_dir, kind).join(id);
    let taken = root(app_dir, kind).join(format!("{id}.taken"));
    std::fs::rename(&dir, &taken).ok()?;
    let path = taken.join(&found.filename);
    Some((Stored { path, ..found }, taken))
}

/// Remove an entry.
pub fn remove(app_dir: &Path, kind: Kind, id: &str) {
    if uuid::Uuid::parse_str(id).is_ok() {
        let _ = std::fs::remove_dir_all(root(app_dir, kind).join(id));
    }
}

fn is_old(dir: &Path) -> bool {
    std::fs::metadata(dir)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age > TTL)
}

fn sweep(app_dir: &Path, kind: Kind) {
    let Ok(entries) = std::fs::read_dir(root(app_dir, kind)) else {
        return;
    };
    for dir in entries.flatten().map(|e| e.path()) {
        if is_old(&dir) {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

/// The uploaded file of `upload_id`, for a tool.
pub fn uploaded(ctx: &McpContext, upload_id: &str) -> Result<Stored, McpError> {
    find(&ctx.app_dir, Kind::Upload, upload_id).ok_or_else(|| {
        McpError::InvalidArguments(format!(
            "no upload '{upload_id}': upload the file to POST {UPLOADS_PATH} (multipart field \
             `file`, DM token) and give the upload_id that it returns. An upload is kept for \
             one hour."
        ))
    })
}

/// Keep `bytes` as a download, for a tool.
pub fn offer(ctx: &McpContext, filename: &str, bytes: &[u8]) -> Result<Value, McpError> {
    let stored = save(&ctx.app_dir, Kind::Download, filename, bytes)
        .map_err(|e| McpError::Internal(format!("cannot keep the download: {e}")))?;
    Ok(download_json(&stored))
}

/// What a tool returns for a download.
pub fn download_json(stored: &Stored) -> Value {
    json!({
        "download_url": format!("{DOWNLOADS_PATH}/{}", stored.id),
        "filename": stored.filename,
        "size_bytes": stored.size_bytes,
        "how": "GET the server address followed by download_url, with the DM token \
                (Authorization: Bearer). The link works once, for one hour.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_name_keeps_the_last_part_and_safe_characters() {
        assert_eq!(clean_name("../../etc/passwd", "x"), "passwd");
        assert_eq!(clean_name("C:\\maps\\Cave 1.uvtt", "x"), "Cave_1.uvtt");
        assert_eq!(clean_name("..", "file"), "file");
        assert_eq!(clean_name(".hidden", "file"), "hidden");
        assert_eq!(clean_name("", "file"), "file");
    }

    #[test]
    fn save_find_remove() {
        let app = tempfile::tempdir().unwrap();
        let s = save(app.path(), Kind::Download, "a b.zip", b"abc").unwrap();
        assert_eq!(s.filename, "a_b.zip");
        let f = find(app.path(), Kind::Download, &s.id).unwrap();
        assert_eq!(f.size_bytes, 3);
        assert_eq!(std::fs::read(&f.path).unwrap(), b"abc");
        // The kinds are apart.
        assert!(find(app.path(), Kind::Upload, &s.id).is_none());
        remove(app.path(), Kind::Download, &s.id);
        assert!(find(app.path(), Kind::Download, &s.id).is_none());
    }

    #[test]
    fn a_download_is_taken_once() {
        let app = tempfile::tempdir().unwrap();
        let s = save(app.path(), Kind::Download, "m.zip", b"abc").unwrap();
        let (got, dir) = take(app.path(), Kind::Download, &s.id).unwrap();
        assert_eq!(std::fs::read(&got.path).unwrap(), b"abc");
        assert!(take(app.path(), Kind::Download, &s.id).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_id_is_never_a_path() {
        let app = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(app.path().join("tmp/secret")).unwrap();
        std::fs::write(app.path().join("tmp/secret/f"), b"x").unwrap();
        assert!(find(app.path(), Kind::Upload, "../secret").is_none());
        remove(app.path(), Kind::Upload, "../secret");
        assert!(app.path().join("tmp/secret/f").exists());
    }
}
