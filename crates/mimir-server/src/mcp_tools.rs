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
        ]
    })
}

/// A tool by name: the registry's, then the server's.
pub fn find(name: &str) -> Option<&'static RegisteredTool> {
    mimir_mcp::registry::find(name).or_else(|| server_tools().iter().find(|t| t.name == name))
}
