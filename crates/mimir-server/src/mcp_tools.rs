//! MCP tools that need the server's own modules (COLLIERY-I-0616): served
//! at `/mcp` beside the mimir-mcp registry's tools.

use std::sync::{Arc, OnceLock};

use mimir_mcp::registry::RegisteredTool;
use mimir_mcp::{tool, tool_args, McpContext, McpError};
use serde_json::Value;

use crate::error::ApiError;

fn mcp_err(e: ApiError) -> McpError {
    McpError::InvalidArguments(e.message)
}

tool_args! {
    pub struct AdventurePlanArgs {
        /// The adventure's key in the reader (its id, e.g. "LMoP")
        pub source: String,
    }
}

tool_args! {
    pub struct AdventureCampaignArgs {
        /// The adventure's key in the reader (its id, e.g. "LMoP")
        pub source: String,
        /// The campaign's name (default: the adventure's)
        pub name: Option<String>,
        /// Choices per part: [{"index": 3, "role": "module" | "document" | "skip", "one_document": false}]. A part not listed takes its role in the plan.
        pub parts: Option<Vec<mimir_wire::PartChoice>>,
    }
}

tool_args! {
    pub struct ExportAdventureArgs {
        /// The adventure's source code: 6 or more letters, digits or - & + !, not an official source (e.g. "LowFen")
        pub source: String,
        /// The adventure's name
        pub name: String,
        /// Its authors, comma-separated
        pub authors: Option<String>,
        /// Its version (default 1.0.0)
        pub version: Option<String>,
        /// Its storyline (default "Homebrew")
        pub storyline: Option<String>,
        /// The lowest character level (default 1)
        pub level_start: Option<i64>,
        /// The highest character level (default: the lowest)
        pub level_end: Option<i64>,
        /// A web address for the adventure
        pub url: Option<String>,
    }
}

/// `export_adventure_file`: the active campaign as a 5etools homebrew
/// adventure, as JSON.
pub async fn export_adventure_file(
    ctx: &Arc<McpContext>,
    args: ExportAdventureArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;
    let q = crate::api::admin::AdventureExportQuery {
        source: args.source,
        name: args.name,
        authors: args.authors.unwrap_or_default(),
        version: args.version,
        url: args.url,
        storyline: args.storyline,
        level_start: args.level_start,
        level_end: args.level_end,
    };
    let mut conn = ctx.connect()?;
    crate::adventure::export_adventure(&mut conn, &campaign_id, &q.meta()).map_err(mcp_err)
}

tool_args! {
    pub struct ExportPackArgs {
        /// The folder to write the pack in
        pub output_path: String,
        /// The adventure's source code: 6 or more letters, digits or - & + !, not an official source
        pub source: String,
        /// The adventure's name
        pub name: String,
        /// Its authors, comma-separated
        pub authors: Option<String>,
        /// The lowest character level (default 1)
        pub level_start: Option<i64>,
        /// The highest character level (default: the lowest)
        pub level_end: Option<i64>,
    }
}

tool_args! {
    pub struct ImportPackArgs {
        /// The path of a .mimir-adventure.zip
        pub path: String,
        /// The new campaign's name (default: the pack's)
        pub name: Option<String>,
    }
}

/// `export_adventure_pack`: the active campaign as a Mimir adventure pack,
/// written to `output_path`.
pub async fn export_adventure_pack(
    ctx: &Arc<McpContext>,
    args: ExportPackArgs,
) -> Result<Value, McpError> {
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;
    let q = crate::api::admin::AdventureExportQuery {
        source: args.source,
        name: args.name,
        authors: args.authors.unwrap_or_default(),
        version: None,
        url: None,
        storyline: None,
        level_start: args.level_start,
        level_end: args.level_end,
    };
    let meta = q.meta();
    let mut conn = ctx.connect()?;
    let bytes = crate::adventure::export_pack(&mut conn, &ctx.app_dir, &campaign_id, &meta)
        .map_err(mcp_err)?;
    let path = std::path::Path::new(&args.output_path)
        .join(format!("{}.mimir-adventure.zip", meta.source));
    std::fs::write(&path, &bytes)
        .map_err(|e| McpError::Internal(format!("{}: {e}", path.display())))?;
    Ok(serde_json::json!({"path": path.display().to_string(), "size_bytes": bytes.len()}))
}

/// `import_adventure_pack`: a pack as a new campaign, which becomes the
/// active one.
pub async fn import_adventure_pack(
    ctx: &Arc<McpContext>,
    args: ImportPackArgs,
) -> Result<Value, McpError> {
    use diesel::Connection;
    let bytes = std::fs::read(&args.path)
        .map_err(|e| McpError::InvalidArguments(format!("{}: {e}", args.path)))?;
    let mut conn = ctx.connect()?;
    let r = conn
        .transaction::<_, ApiError, _>(|c| {
            crate::adventure::import_pack(c, &ctx.app_dir, &bytes, args.name.as_deref())
        })
        .map_err(mcp_err)?;
    ctx.set_active_campaign_id(Some(r.campaign_id.clone()));
    Ok(serde_json::json!({"campaign_id": r.campaign_id, "campaign_name": r.campaign_name}))
}

tool_args! {
    pub struct PrintBookArgs {
        /// The folder to write the PDF in
        pub output_path: String,
        /// Its authors, comma-separated, for the title page
        pub authors: Option<String>,
        /// The lowest character level, for the title page
        pub level_start: Option<i64>,
        /// The highest character level, for the title page
        pub level_end: Option<i64>,
    }
}

/// `print_adventure_book`: the active campaign as one PDF book.
pub async fn print_adventure_book(
    ctx: &Arc<McpContext>,
    args: PrintBookArgs,
) -> Result<Value, McpError> {
    use base64::Engine;
    let campaign_id = ctx
        .get_active_campaign_id()
        .ok_or(McpError::NoActiveCampaign)?;
    let print_ctx = crate::print::PrintCtx::new(ctx.db_url().to_string(), ctx.app_dir.clone());
    let print_state = mimir_print::PrintState {
        templates_dir: ctx.app_dir.join("templates"),
        assets_dir: ctx.app_dir.join("assets"),
    };
    let r = crate::print::export_campaign_book(
        &print_ctx,
        &print_state,
        campaign_id.clone(),
        Some(crate::print::BookOptions {
            authors: args.authors,
            level_start: args.level_start,
            level_end: args.level_end,
        }),
    );
    let data = r
        .data
        .ok_or_else(|| McpError::Internal(r.error.unwrap_or_else(|| "no PDF".into())))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.pdf_base64)
        .map_err(|e| McpError::Internal(e.to_string()))?;
    let path = std::path::Path::new(&args.output_path).join(format!("book-{campaign_id}.pdf"));
    std::fs::write(&path, &bytes)
        .map_err(|e| McpError::Internal(format!("{}: {e}", path.display())))?;
    Ok(serde_json::json!({"path": path.display().to_string(), "size_bytes": bytes.len()}))
}

/// `preview_adventure_campaign`
pub async fn preview_adventure_campaign(
    ctx: &Arc<McpContext>,
    args: AdventurePlanArgs,
) -> Result<Value, McpError> {
    let mut conn = ctx.connect()?;
    let a = crate::adventure::adventure_text(&mut conn, &args.source).map_err(mcp_err)?;
    serde_json::to_value(crate::adventure::plan_view(&a))
        .map_err(|e| McpError::Internal(e.to_string()))
}

/// `create_campaign_from_adventure`: the new campaign becomes the active one.
pub async fn create_campaign_from_adventure(
    ctx: &Arc<McpContext>,
    args: AdventureCampaignArgs,
) -> Result<Value, McpError> {
    let mut conn = ctx.connect()?;
    let a = crate::adventure::adventure_text(&mut conn, &args.source).map_err(mcp_err)?;
    let req = mimir_wire::AdventureCampaignRequest {
        name: args.name.unwrap_or_default(),
        parts: args.parts.unwrap_or_default(),
    };
    let made =
        crate::adventure::create_campaign(&mut conn, &ctx.app_dir, &a, &req).map_err(mcp_err)?;
    ctx.set_active_campaign_id(Some(made.campaign_id.clone()));
    serde_json::to_value(made).map_err(|e| McpError::Internal(e.to_string()))
}

/// The server's tools.
pub fn server_tools() -> &'static [RegisteredTool] {
    static TOOLS: OnceLock<Vec<RegisteredTool>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        vec![
            tool!(
                "preview_adventure_campaign",
                "Show how an adventure in the reader becomes a campaign: each part (chapter, appendix) with its role (module or campaign document), its documents, its square-grid maps and how many creatures it names.",
                AdventurePlanArgs,
                preview_adventure_campaign
            ),
            tool!(
                "create_campaign_from_adventure",
                "Make a campaign from an adventure in the reader: modules with their documents, maps and monsters, and campaign documents. The new campaign becomes the active campaign.",
                AdventureCampaignArgs,
                create_campaign_from_adventure
            ),
            tool!(
                "export_adventure_file",
                "Export the active campaign as a 5etools homebrew adventure file (JSON): its documents, its modules as chapters, its homebrew, and a copy of every catalog entry the text names under the adventure's own source. 5etools and Mimir (Settings, Books, Import a homebrew file) read it.",
                ExportAdventureArgs,
                export_adventure_file
            ),
            tool!(
                "export_adventure_pack",
                "Write the active campaign as a Mimir adventure pack (.mimir-adventure.zip) in a folder: the adventure to be played, with its maps, walls, placements, module monsters and NPCs, for another Mimir. No player characters, notes, fog or combat state.",
                ExportPackArgs,
                export_adventure_pack
            ),
            tool!(
                "import_adventure_pack",
                "Import a Mimir adventure pack (.mimir-adventure.zip) as a new campaign; it becomes the active campaign.",
                ImportPackArgs,
                import_adventure_pack
            ),
            tool!(
                "print_adventure_book",
                "Write the active campaign as one PDF book in a folder: a title page (authors, levels), contents, the campaign documents, each module with its documents and maps, the appendices, and the stat blocks and items the text names.",
                PrintBookArgs,
                print_adventure_book
            ),
        ]
    })
}

/// A tool by name: the registry's, then the server's.
pub fn find(name: &str) -> Option<&'static RegisteredTool> {
    mimir_mcp::registry::find(name).or_else(|| server_tools().iter().find(|t| t.name == name))
}
