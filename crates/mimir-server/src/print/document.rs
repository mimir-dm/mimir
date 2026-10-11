//! Document Export Commands
//!
//! The server's PDF routes: exporting campaign and module documents to PDF.

use base64::Engine;
use mimir_core::dal::campaign as dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::services::{CampaignService, CharacterService, DocumentService, MapService};
use mimir_print::sections::{
    CharacterData, CharacterSection, ClassInfo, CutoutToken, MapPreview, MonsterCardSection,
    Proficiencies, ProficiencyEntry, TiledMapSection, TokenCutoutSection,
};
use mimir_print::{DocumentBuilder, MarkdownSection, PrintState};
use serde_json::Value;
use tracing::{error, info};

use super::PrintCtx;

use super::helpers::{compute_ac, compute_hit_die_string, compute_hp_max, enrich_inventory_item};
use super::{ApiResponse, CampaignExportOptions, ModuleExportOptions, PrintResult};

/// Look up monster data from catalog or homebrew, returning parsed JSON.
fn lookup_monster_data(
    db: &mut diesel::SqliteConnection,
    monster_name: &str,
    monster_source: &str,
    campaign_id: &str,
) -> Option<Value> {
    if monster_source == "HB" {
        lookup_homebrew_monster_by_name(db, campaign_id, monster_name)
    } else {
        lookup_catalog_monster(db, monster_name, monster_source)
    }
}

fn lookup_homebrew_monster_by_name(
    db: &mut diesel::SqliteConnection,
    campaign_id: &str,
    monster_name: &str,
) -> Option<Value> {
    match dal::get_campaign_homebrew_monster_by_name(db, campaign_id, monster_name) {
        Ok(Some(hb_monster)) => parse_homebrew_monster_data(hb_monster),
        Ok(None) => {
            error!("Homebrew monster not found: {}", monster_name);
            None
        }
        Err(e) => {
            error!("Failed to look up homebrew monster {}: {}", monster_name, e);
            None
        }
    }
}

fn lookup_homebrew_monster_by_id(
    db: &mut diesel::SqliteConnection,
    homebrew_monster_id: &str,
) -> Option<Value> {
    match dal::get_campaign_homebrew_monster(db, homebrew_monster_id) {
        Ok(hb_monster) => parse_homebrew_monster_data(hb_monster),
        Err(e) => {
            error!(
                "Failed to look up homebrew monster {}: {}",
                homebrew_monster_id, e
            );
            None
        }
    }
}

fn parse_homebrew_monster_data(
    hb_monster: mimir_core::models::campaign::CampaignHomebrewMonster,
) -> Option<Value> {
    match serde_json::from_str::<Value>(&hb_monster.data) {
        Ok(mut data) => {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("name".to_string(), Value::String(hb_monster.name));
                obj.insert("source".to_string(), Value::String("Homebrew".to_string()));
            }
            Some(data)
        }
        Err(e) => {
            error!("Failed to parse homebrew monster data: {}", e);
            None
        }
    }
}

fn lookup_catalog_monster(
    db: &mut diesel::SqliteConnection,
    monster_name: &str,
    monster_source: &str,
) -> Option<Value> {
    match catalog_dal::get_monster_by_name(db, monster_name, monster_source) {
        Ok(Some(catalog_monster)) => match catalog_monster.parse_data() {
            Ok(data) => Some(data),
            Err(e) => {
                error!("Failed to parse monster data for {}: {}", monster_name, e);
                None
            }
        },
        Ok(None) => {
            error!(
                "Catalog monster not found: {} ({})",
                monster_name, monster_source
            );
            None
        }
        Err(e) => {
            error!("Failed to look up monster {}: {}", monster_name, e);
            None
        }
    }
}

/// Resolve monster data for a module monster, handling both catalog and homebrew paths.
fn resolve_module_monster_data(
    db: &mut diesel::SqliteConnection,
    mm: &mimir_core::models::campaign::ModuleMonster,
    campaign_id: &str,
) -> Option<Value> {
    if let Some(ref hb_id) = mm.homebrew_monster_id {
        lookup_homebrew_monster_by_id(db, hb_id)
    } else if let (Some(ref name), Some(ref source)) = (&mm.monster_name, &mm.monster_source) {
        lookup_monster_data(db, name, source, campaign_id)
    } else {
        error!(
            "Module monster {} has no catalog or homebrew reference",
            mm.id
        );
        None
    }
}

/// Resolve monster size from catalog or homebrew.
fn resolve_monster_size(
    db: &mut diesel::SqliteConnection,
    mm: &mimir_core::models::campaign::ModuleMonster,
    campaign_id: &str,
) -> String {
    if let Some(ref hb_id) = mm.homebrew_monster_id {
        // Homebrew monster — get size from homebrew data
        match dal::get_campaign_homebrew_monster(db, hb_id) {
            Ok(hb) => serde_json::from_str::<Value>(&hb.data)
                .ok()
                .and_then(|d| {
                    d.get("size").and_then(|s| s.as_str()).map(|s| {
                        match s {
                            "T" => "Tiny",
                            "S" => "Small",
                            "M" => "Medium",
                            "L" => "Large",
                            "H" => "Huge",
                            "G" => "Gargantuan",
                            other => other,
                        }
                        .to_string()
                    })
                })
                .unwrap_or_else(|| "Medium".to_string()),
            _ => "Medium".to_string(),
        }
    } else if let (Some(ref name), Some(ref source)) = (&mm.monster_name, &mm.monster_source) {
        if source == "HB" {
            // Legacy: homebrew by name
            match dal::get_campaign_homebrew_monster_by_name(db, campaign_id, name) {
                Ok(Some(hb)) => serde_json::from_str::<Value>(&hb.data)
                    .ok()
                    .and_then(|d| {
                        d.get("size").and_then(|s| s.as_str()).map(|s| {
                            match s {
                                "T" => "Tiny",
                                "S" => "Small",
                                "M" => "Medium",
                                "L" => "Large",
                                "H" => "Huge",
                                "G" => "Gargantuan",
                                other => other,
                            }
                            .to_string()
                        })
                    })
                    .unwrap_or_else(|| "Medium".to_string()),
                _ => "Medium".to_string(),
            }
        } else {
            match catalog_dal::get_monster_by_name(db, name, source) {
                Ok(Some(catalog_monster)) => {
                    catalog_monster.size.unwrap_or_else(|| "Medium".to_string())
                }
                _ => "Medium".to_string(),
            }
        }
    } else {
        "Medium".to_string()
    }
}

/// Load token image for a catalog monster (homebrew monsters don't have catalog images).
fn load_monster_token_image(
    mm: &mimir_core::models::campaign::ModuleMonster,
    assets_dir: &std::path::Path,
) -> Option<Vec<u8>> {
    let (name, source) = match (&mm.monster_name, &mm.monster_source) {
        (Some(n), Some(s)) if s != "HB" => (n, s),
        _ => return None, // Homebrew monsters don't have catalog token images
    };

    let img_base = assets_dir
        .join("catalog")
        .join("bestiary")
        .join("tokens")
        .join(source);

    let extensions = ["webp", "png", "jpg", "jpeg"];
    for ext in &extensions {
        let path = img_base.join(format!("{}.{}", name, ext));
        if path.exists() {
            match std::fs::read(&path) {
                Ok(bytes) => return Some(bytes),
                Err(e) => {
                    error!("Failed to read token image {:?}: {}", path, e);
                }
            }
        }
    }
    None
}

/// Export a single campaign document to PDF
/// A document's markdown as a section, its stat block lines from the
/// catalog (MIMIR-T-0737).
fn doc_section(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    content: &str,
) -> mimir_print::Result<MarkdownSection> {
    let db = std::cell::RefCell::new(db);
    let resolve = |tag: &str, name: &str, source: Option<&str>| {
        statblock_typst(&mut db.borrow_mut(), tag, name, source)
    };
    let image = |url: &str| image_bytes(&mut db.borrow_mut(), app_dir, url);
    MarkdownSection::from_markdown_full(content, &resolve, &image)
}

/// The bytes of an image a document names (MIMIR-T-0742): a 5etools image
/// (`5etools:<path>`, under the catalog images) or an uploaded image
/// (`/api/v1/assets/<id>/file`). Another address prints as its title.
pub fn image_bytes(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    url: &str,
) -> Option<Vec<u8>> {
    if let Some(path) = url.strip_prefix(mimir_render::adventure::IMAGE_SCHEME) {
        let path = crate::api::reference::safe_image_path(path)?;
        return std::fs::read(app_dir.join("assets").join("catalog").join(path)).ok();
    }
    let id = url.strip_prefix("/api/v1/assets/")?.strip_suffix("/file")?;
    let asset = mimir_core::dal::campaign::get_campaign_asset(db, id).ok()?;
    std::fs::read(app_dir.join(&asset.blob_path)).ok()
}

/// Typst for a stat block line: a creature's stat block, else the entry's
/// name and text (a creature's lore for `{@creatureFluff}`). `None` when
/// the catalog does not have it.
fn statblock_typst(
    db: &mut diesel::SqliteConnection,
    tag: &str,
    name: &str,
    source: Option<&str>,
) -> Option<String> {
    use mimir_print::sections::card_utils::flatten_entries;
    use mimir_print::sections::monster_cards::{monster_statblock_typst, plain_tags};
    let (kind, lore) = if tag == "creatureFluff" {
        ("creature", true)
    } else {
        (tag, false)
    };
    let category = crate::adventure::category_of(kind)?;
    let v = crate::api::reference::entry_named(db, category, name, source.unwrap_or(""))
        .ok()
        .flatten()?;
    if category == "monsters" && !lore {
        return Some(monster_statblock_typst(&v));
    }
    let entries = if lore {
        v.pointer("/fluff/entries")
    } else {
        v.get("entries")
    };
    let text = entries
        .and_then(Value::as_array)
        .map(|l| flatten_entries(l))
        .unwrap_or_default();
    let lit = |s: &str| format!("#\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    Some(format!(
        "#block(width: 100%, inset: 8pt, radius: 3pt, stroke: 0.5pt + luma(150))[*{}* \\\n{}]\n\n",
        lit(v["name"].as_str().unwrap_or(name)),
        lit(&plain_tags(&text))
    ))
}

pub fn export_campaign_document(
    app_state: &PrintCtx,
    print_state: &PrintState,
    document_id: String,
) -> ApiResponse<PrintResult> {
    info!("Exporting campaign document {} to PDF", document_id);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the document
    let document = match DocumentService::new(&mut db).get(&document_id) {
        Ok(Some(doc)) => doc,
        Ok(None) => return ApiResponse::err(format!("Document not found: {}", document_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get document: {}", e)),
    };

    // Get the campaign for the title
    let campaign = match CampaignService::new(&mut db).get(&document.campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return ApiResponse::err(format!("Campaign not found: {}", document.campaign_id))
        }
        Err(e) => return ApiResponse::err(format!("Failed to get campaign: {}", e)),
    };

    // Build the PDF using markdown section
    let section = match doc_section(&mut db, &app_state.paths.app_dir, &document.content) {
        Ok(s) => s.with_title(&document.title),
        Err(e) => return ApiResponse::err(format!("Failed to parse markdown: {}", e)),
    };

    let pdf_result = DocumentBuilder::new(&campaign.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Campaign document PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate campaign document PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// The title page's lines of a book: its authors and its levels.
pub fn book_lines(o: &super::BookOptions) -> Vec<String> {
    let mut lines = Vec::new();
    let authors: Vec<&str> = o
        .authors
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .collect();
    if !authors.is_empty() {
        lines.push(format!("By {}", authors.join(", ")));
    }
    match (o.level_start, o.level_end) {
        (Some(a), Some(b)) if a != b => lines.push(format!("For characters of levels {a} to {b}")),
        (Some(a), _) => lines.push(format!("For characters of level {a}")),
        _ => {}
    }
    lines
}

/// The campaign as one book (MIMIR-T-0742): a title page and contents, the
/// campaign documents, each module (its documents, then the maps that no
/// document shows), the appendices ("Appendix …", "Credits"), and an
/// appendix with the stat blocks and items the text names.
pub fn export_campaign_book(
    app_state: &PrintCtx,
    print_state: &PrintState,
    campaign_id: String,
    options: Option<super::BookOptions>,
) -> ApiResponse<PrintResult> {
    let opts = options.unwrap_or_default();
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };
    let campaign = match CampaignService::new(&mut db).get(&campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => return ApiResponse::err(format!("Campaign not found: {}", campaign_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get campaign: {}", e)),
    };
    use mimir_core::dal::campaign as dal;
    let app_dir = app_state.paths.app_dir.clone();
    let docs = match dal::list_campaign_level_documents(&mut db, &campaign_id) {
        Ok(d) => d,
        Err(e) => return ApiResponse::err(format!("Failed to list documents: {}", e)),
    };
    let modules = match dal::list_modules(&mut db, &campaign_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(format!("Failed to list modules: {}", e)),
    };
    let is_back = |t: &str| {
        let t = t.to_lowercase();
        t.starts_with("appendix") || t == "credits"
    };

    let mut builder = DocumentBuilder::new(&campaign.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(true)
        .with_title_lines(book_lines(&opts))
        .with_toc(true);
    let mut all_text = String::new();
    let mut creatures: std::collections::BTreeSet<mimir_render::adventure::Ref> =
        Default::default();

    for d in docs.iter().filter(|d| !is_back(&d.title)) {
        all_text.push_str(&d.content);
        match doc_section(&mut db, &app_dir, &d.content) {
            Ok(s) => builder = builder.append(s.with_title(&d.title)),
            Err(e) => return ApiResponse::err(format!("{}: {}", d.title, e)),
        }
    }
    for m in &modules {
        let mdocs = dal::list_module_documents(&mut db, &m.id).unwrap_or_default();
        let mut content = String::new();
        for d in &mdocs {
            all_text.push_str(&d.content);
            content.push_str(&format!("\n\n## {}\n\n{}\n", d.title, d.content));
        }
        // The module's maps that no document already shows (an adventure's
        // map image has the map's name as its title).
        let maps = dal::list_module_maps(&mut db, &m.id).unwrap_or_default();
        let shown: Vec<&mimir_core::models::campaign::Map> = maps
            .iter()
            .filter(|map| !content.contains(&format!("![{}](", map.name)))
            .collect();
        if !shown.is_empty() {
            content.push_str("\n\n## Maps\n\n");
            for map in shown {
                content.push_str(&format!("![{}](mimir-map:{})\n\n", map.name, map.id));
            }
        }
        for mm in dal::list_module_monsters(&mut db, &m.id).unwrap_or_default() {
            if let (Some(name), Some(source)) = (mm.monster_name, mm.monster_source) {
                creatures.insert(mimir_render::adventure::Ref {
                    kind: "creature".into(),
                    name,
                    source,
                });
            }
        }
        match book_section(&mut db, &app_dir, &content) {
            Ok(s) => builder = builder.append(s.with_title(&m.name)),
            Err(e) => return ApiResponse::err(format!("{}: {}", m.name, e)),
        }
    }
    for d in docs.iter().filter(|d| is_back(&d.title)) {
        all_text.push_str(&d.content);
        match doc_section(&mut db, &app_dir, &d.content) {
            Ok(s) => builder = builder.append(s.with_title(&d.title)),
            Err(e) => return ApiResponse::err(format!("{}: {}", d.title, e)),
        }
    }

    // The stat blocks and items the text and the modules name.
    let mut named = std::collections::BTreeSet::new();
    mimir_render::adventure::text_refs(&all_text, &mut named);
    named.extend(creatures);
    let named: std::collections::BTreeSet<_> = named
        .into_iter()
        .filter(|r| r.kind == "creature" || r.kind == "item")
        .collect();
    if !named.is_empty() {
        let found = match crate::adventure::closure(&mut db, named.clone()) {
            Ok(f) => f,
            Err(e) => return ApiResponse::err(e.message),
        };
        let mut entries: Vec<&mimir_render::adventure::Ref> = found
            .entries
            .keys()
            .filter(|r| {
                (r.kind == "creature" || r.kind == "item")
                    && found
                        .resolved
                        .iter()
                        .any(|(from, to)| named.contains(from) && to == *r)
            })
            .collect();
        entries.sort_by(|a, b| {
            (&a.kind, a.name.to_lowercase()).cmp(&(&b.kind, b.name.to_lowercase()))
        });
        if !entries.is_empty() {
            let lines: String = entries
                .iter()
                .map(|r| format!("{{@{} {}|{}}}\n\n", r.kind, r.name, r.source))
                .collect();
            match doc_section(&mut db, &app_dir, &lines) {
                Ok(s) => builder = builder.append(s.with_title("Appendix: Stat Blocks and Items")),
                Err(e) => return ApiResponse::err(e.to_string()),
            }
        }
    }

    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            ApiResponse::ok(PrintResult {
                pdf_base64: base64::engine::general_purpose::STANDARD.encode(&pdf_bytes),
                size_bytes,
            })
        }
        Err(e) => ApiResponse::err(format!("Failed to make the book: {}", e)),
    }
}

/// A book section: [`doc_section`], with module maps (`mimir-map:<id>`)
/// printed from their files.
fn book_section(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    content: &str,
) -> mimir_print::Result<MarkdownSection> {
    let db = std::cell::RefCell::new(db);
    let resolve = |tag: &str, name: &str, source: Option<&str>| {
        statblock_typst(&mut db.borrow_mut(), tag, name, source)
    };
    let image = |url: &str| match url.strip_prefix("mimir-map:") {
        Some(map_id) => {
            let conn = &mut *db.borrow_mut();
            let map = mimir_core::dal::campaign::get_map(conn, map_id).ok()?;
            let asset =
                mimir_core::dal::campaign::get_campaign_asset(conn, &map.uvtt_asset_id).ok()?;
            mimir_print::map_renderer::load_image_from_file(&app_dir.join(&asset.blob_path)).ok()
        }
        None => image_bytes(&mut db.borrow_mut(), app_dir, url),
    };
    MarkdownSection::from_markdown_full(content, &resolve, &image)
}

/// Export all campaign documents to PDF
pub fn export_campaign_documents(
    app_state: &PrintCtx,
    print_state: &PrintState,
    campaign_id: String,
    options: Option<CampaignExportOptions>,
) -> ApiResponse<PrintResult> {
    use mimir_core::services::ModuleService;

    info!("Exporting campaign {} to PDF", campaign_id);

    let opts = options.unwrap_or_default();

    // Log received options
    info!("=== Campaign Export Options ===");
    info!("  include_campaign_docs: {:?}", opts.include_campaign_docs);
    info!(
        "  include_module_content: {:?}",
        opts.include_module_content
    );
    info!("  include_npcs: {:?}", opts.include_npcs);
    info!(
        "  include_module_map_previews: {:?}",
        opts.include_module_map_previews
    );
    info!(
        "  include_module_tiled_maps: {:?}",
        opts.include_module_tiled_maps
    );
    info!("  include_token_cutouts: {:?}", opts.include_token_cutouts);
    info!(
        "  include_campaign_map_previews: {:?}",
        opts.include_campaign_map_previews
    );
    info!(
        "  include_campaign_tiled_maps: {:?}",
        opts.include_campaign_tiled_maps
    );
    info!("================================");

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the campaign
    let campaign = match CampaignService::new(&mut db).get(&campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => return ApiResponse::err(format!("Campaign not found: {}", campaign_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get campaign: {}", e)),
    };

    // Build combined PDF
    let mut builder = DocumentBuilder::new(&campaign.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(true)
        .with_toc(true);

    let mut has_content = false;

    // 1. Campaign-level documents
    if opts.include_campaign_docs.unwrap_or(true) {
        info!("[SECTION] Campaign documents requested");
        let documents = match DocumentService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(docs) => {
                info!("  Found {} campaign documents", docs.len());
                docs
            }
            Err(e) => {
                error!("  Failed to list campaign documents: {}", e);
                vec![]
            }
        };

        for doc in documents {
            match doc_section(&mut db, &app_state.paths.app_dir, &doc.content) {
                Ok(section) => {
                    info!("  Adding document: {}", doc.title);
                    builder = builder.append(section.with_title(&doc.title));
                    has_content = true;
                }
                Err(e) => {
                    error!("  Failed to parse document {}: {}", doc.id, e);
                }
            }
        }
    } else {
        info!("[SECTION] Campaign documents NOT requested");
    }

    // 2. Module content (documents + monsters)
    if opts.include_module_content.unwrap_or(false) {
        info!("[SECTION] Module content requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list modules: {}", e);
                vec![]
            }
        };

        for module in modules {
            info!("  Processing module: {}", module.name);

            // Module documents
            let module_docs = match DocumentService::new(&mut db).list_for_module(&module.id) {
                Ok(docs) => {
                    info!("    Found {} documents", docs.len());
                    docs
                }
                Err(e) => {
                    error!("    Failed to list module {} documents: {}", module.id, e);
                    vec![]
                }
            };

            for doc in module_docs {
                match doc_section(&mut db, &app_state.paths.app_dir, &doc.content) {
                    Ok(section) => {
                        let title = format!("{}: {}", module.name, doc.title);
                        info!("    Adding document: {}", title);
                        builder = builder.append(section.with_title(&title));
                        has_content = true;
                    }
                    Err(e) => {
                        error!("    Failed to parse document {}: {}", doc.id, e);
                    }
                }
            }

            // Module monsters as cards
            let module_monsters = match dal::list_module_monsters(&mut db, &module.id) {
                Ok(monsters) => {
                    info!("    Found {} monsters", monsters.len());
                    monsters
                }
                Err(e) => {
                    error!("    Failed to list module {} monsters: {}", module.id, e);
                    vec![]
                }
            };

            if !module_monsters.is_empty() {
                let mut monster_data: Vec<Value> = Vec::new();
                for mm in &module_monsters {
                    if let Some(mut data) = resolve_module_monster_data(&mut db, mm, &campaign_id) {
                        if let Some(ref display_name) = mm.display_name {
                            if let Some(obj) = data.as_object_mut() {
                                obj.insert("name".to_string(), Value::String(display_name.clone()));
                            }
                        }
                        for _ in 0..mm.quantity {
                            monster_data.push(data.clone());
                        }
                    }
                }

                if !monster_data.is_empty() {
                    info!(
                        "    Adding MonsterCardSection with {} monsters",
                        monster_data.len()
                    );
                    let section = MonsterCardSection::new(monster_data);
                    builder = builder.append(section);
                    has_content = true;
                }
            }

            // Module traps as cards (from map_traps table)
            {
                use mimir_print::sections::TrapCardSection;
                use std::collections::HashSet;

                let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
                let maps = match map_service.list_for_module(&module.id) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("    Failed to list module maps for traps: {}", e);
                        vec![]
                    }
                };

                let mut seen_traps: HashSet<String> = HashSet::new();
                let mut trap_data: Vec<Value> = Vec::new();

                for map in &maps {
                    if let Ok(map_traps) = dal::list_map_traps(&mut db, &map.id) {
                        for trap in map_traps {
                            if seen_traps.contains(&trap.name) {
                                continue;
                            }
                            seen_traps.insert(trap.name.clone());

                            // Try catalog lookup
                            let sources_to_try = ["DMG", "XGE", "TCE", "PHB"];
                            let mut found_catalog = false;

                            for source in sources_to_try {
                                if let Ok(Some(catalog_trap)) =
                                    catalog_dal::get_trap_by_name(&mut db, &trap.name, source)
                                {
                                    if let Ok(data) = catalog_trap.parse_data() {
                                        trap_data.push(data);
                                        found_catalog = true;
                                        break;
                                    }
                                }
                            }

                            // Create custom trap card if not in catalog
                            if !found_catalog {
                                let custom_trap = serde_json::json!({
                                    "name": trap.name,
                                    "trapHazType": "TRAP",
                                    "effect": [
                                        {
                                            "type": "entries",
                                            "name": "Trigger",
                                            "entries": [trap.trigger_description.as_deref().unwrap_or("Unknown trigger")]
                                        },
                                        {
                                            "type": "entries",
                                            "name": "Effect",
                                            "entries": [trap.effect_description.as_deref().unwrap_or("Unknown effect")]
                                        }
                                    ],
                                    "countermeasures": [trap.description.as_deref().unwrap_or("No countermeasures listed")],
                                    "dc": trap.dc
                                });
                                trap_data.push(custom_trap);
                            }
                        }
                    }
                }

                if !trap_data.is_empty() {
                    info!("    Adding TrapCardSection with {} traps", trap_data.len());
                    let section = TrapCardSection::new(trap_data);
                    builder = builder.append(section);
                    has_content = true;
                }
            }

            // Module POIs (from map_pois table)
            {
                let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
                let maps = match map_service.list_for_module(&module.id) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("    Failed to list module maps for POIs: {}", e);
                        vec![]
                    }
                };

                let mut poi_content = String::new();
                let mut total_pois = 0;

                for map in &maps {
                    if let Ok(map_pois) = dal::list_map_pois(&mut db, &map.id) {
                        if !map_pois.is_empty() {
                            poi_content.push_str(&format!("### {}\n\n", map.name));
                            for poi in &map_pois {
                                total_pois += 1;
                                poi_content.push_str(&format!(
                                    "**{}** ({},{})\n",
                                    poi.name, poi.grid_x, poi.grid_y
                                ));
                                if let Some(ref desc) = poi.description {
                                    poi_content.push_str(&format!("{}\n", desc));
                                }
                                poi_content.push('\n');
                            }
                        }
                    }
                }

                if !poi_content.is_empty() {
                    info!(
                        "    Adding POI section with {} points of interest",
                        total_pois
                    );
                    let section_title = format!("{}: Points of Interest", module.name);
                    let section = MarkdownSection::from_content(&poi_content, Some(&section_title));
                    builder = builder.append(section);
                    has_content = true;
                }
            }
        }
    } else {
        info!("[SECTION] Module content NOT requested");
    }

    // 3. NPCs (characters marked as NPCs)
    if opts.include_npcs.unwrap_or(false) {
        info!("[SECTION] NPCs requested");
        let npcs = match CharacterService::new(&mut db).list_npcs(&campaign_id) {
            Ok(n) => {
                info!("  Found {} NPCs", n.len());
                n
            }
            Err(e) => {
                error!("  Failed to list NPCs: {}", e);
                vec![]
            }
        };

        for npc in npcs {
            info!("  Processing NPC: {}", npc.name);
            let classes = dal::list_character_classes(&mut db, &npc.id).unwrap_or_default();
            let inventory = CharacterService::new(&mut db)
                .get_inventory(&npc.id)
                .unwrap_or_default();
            let proficiencies_raw =
                dal::list_character_proficiencies(&mut db, &npc.id).unwrap_or_default();

            // Build proficiencies struct
            let proficiencies = {
                let mut profs = Proficiencies::default();
                for p in proficiencies_raw {
                    let expertise = p.has_expertise();
                    match p.proficiency_type.as_str() {
                        "skill" => profs.skills.push(ProficiencyEntry {
                            name: p.name,
                            expertise,
                        }),
                        "save" => profs.saves.push(p.name),
                        "language" => profs.languages.push(p.name),
                        "armor" => profs.armor.push(p.name),
                        "weapon" => profs.weapons.push(p.name),
                        "tool" => profs.tools.push(p.name),
                        _ => {}
                    }
                }
                profs
            };

            let mut char_data = CharacterData {
                name: npc.name.clone(),
                player_name: npc.player_name.clone(),
                is_npc: npc.is_npc(),
                race_name: npc.race_name.clone(),
                background_name: npc.background_name.clone(),
                strength: npc.strength,
                dexterity: npc.dexterity,
                constitution: npc.constitution,
                intelligence: npc.intelligence,
                wisdom: npc.wisdom,
                charisma: npc.charisma,
                cp: npc.cp,
                sp: npc.sp,
                ep: npc.ep,
                gp: npc.gp,
                pp: npc.pp,
                traits: npc.traits.clone(),
                ideals: npc.ideals.clone(),
                bonds: npc.bonds.clone(),
                flaws: npc.flaws.clone(),
                role: npc.role.clone(),
                location: npc.location.clone(),
                faction: npc.faction.clone(),
                classes: classes
                    .into_iter()
                    .map(|c| {
                        let is_starting = c.is_starting_class();
                        ClassInfo {
                            class_name: c.class_name,
                            class_source: c.class_source,
                            level: c.level,
                            subclass_name: c.subclass_name,
                            is_starting,
                        }
                    })
                    .collect(),
                inventory: inventory
                    .iter()
                    .map(|i| enrich_inventory_item(&mut db, i, Some(&campaign_id)))
                    .collect(),
                proficiencies,
                speed: 30,
                ac: 10, // computed below
                hit_points_max: 0,
                hit_die: String::new(),
                spellcasting_ability: None,
                spell_save_dc: None,
                spell_attack_bonus: None,
                spell_slots: vec![0; 9],
            };

            // Compute AC from equipped armor
            let npc_dex_mod = (npc.dexterity - 10).div_euclid(2);
            char_data.ac = compute_ac(&char_data.inventory, npc_dex_mod);

            // Compute HP and hit dice for NPC
            let con_mod = (npc.constitution - 10).div_euclid(2);
            char_data.hit_points_max = compute_hp_max(&char_data.classes, con_mod);
            char_data.hit_die = compute_hit_die_string(&char_data.classes);

            info!("    Adding CharacterSection for NPC: {}", char_data.name);
            builder = builder.append(CharacterSection::new(char_data));
            has_content = true;
        }
    } else {
        info!("[SECTION] NPCs NOT requested");
    }

    // 4. Campaign-level map previews
    if opts.include_campaign_map_previews.unwrap_or(false) {
        info!("[SECTION] Campaign map previews requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_campaign_level(&campaign_id) {
            Ok(m) => {
                info!("  Found {} campaign-level maps", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list campaign maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                info!("  Adding map preview: {}", map.name);
                                let preview =
                                    MapPreview::from_rendered(map.name.clone(), image_bytes);
                                builder = builder.append(preview);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("  Failed to decode map image for {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Campaign map previews NOT requested");
    }

    // 5. Module map previews
    if opts.include_module_map_previews.unwrap_or(false) {
        info!("[SECTION] Module map previews requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules to check for maps", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list modules: {}", e);
                vec![]
            }
        };

        for module in modules {
            let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
            let maps = match map_service.list_for_module(&module.id) {
                Ok(m) => {
                    info!("  Module '{}' has {} maps", module.name, m.len());
                    m
                }
                Err(e) => {
                    error!("  Failed to list module {} maps: {}", module.id, e);
                    vec![]
                }
            };

            for map in maps {
                if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                    if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                        // Extract base64 image from UVTT and decode it
                        if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str())
                        {
                            match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                                Ok(image_bytes) => {
                                    let name = format!("{}: {}", module.name, map.name);
                                    info!("    Adding map preview: {}", name);
                                    let preview = MapPreview::from_rendered(name, image_bytes);
                                    builder = builder.append(preview);
                                    has_content = true;
                                }
                                Err(e) => {
                                    error!(
                                        "    Failed to decode map image for {}: {}",
                                        map.name, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Module map previews NOT requested");
    }

    // 6. Campaign tiled maps (for tabletop play at 1"=5ft scale)
    if opts.include_campaign_tiled_maps.unwrap_or(false) {
        info!("[SECTION] Campaign tiled maps requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_campaign_level(&campaign_id) {
            Ok(m) => {
                info!("  Found {} campaign-level maps for tiled export", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list campaign maps for tiled export: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    let pixels_per_grid = uvtt_json
                        .get("resolution")
                        .and_then(|r| r.get("pixels_per_grid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(70) as i32;

                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                let tiled = TiledMapSection::from_rendered(
                                    map.name.clone(),
                                    image_bytes,
                                    pixels_per_grid,
                                );
                                builder = builder.append(tiled);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("Failed to decode map image for tiled {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    // 7. Module tiled maps (for tabletop play at 1"=5ft scale)
    if opts.include_module_tiled_maps.unwrap_or(false) {
        info!("[SECTION] Module tiled maps requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules for tiled maps", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list modules for tiled maps: {}", e);
                vec![]
            }
        };

        for module in modules {
            let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
            let maps = match map_service.list_for_module(&module.id) {
                Ok(m) => {
                    info!(
                        "  Module '{}' has {} maps for tiled export",
                        module.name,
                        m.len()
                    );
                    m
                }
                Err(e) => {
                    error!(
                        "  Failed to list module {} maps for tiled export: {}",
                        module.id, e
                    );
                    vec![]
                }
            };

            for map in maps {
                if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                    if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                        let resolution = uvtt_json.get("resolution");
                        let pixels_per_grid = resolution
                            .and_then(|r| r.get("pixels_per_grid"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(70) as i32;

                        // Extract base64 image from UVTT and decode it
                        if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str())
                        {
                            match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                                Ok(image_bytes) => {
                                    let name = format!("{}: {}", module.name, map.name);
                                    let tiled = TiledMapSection::from_rendered(
                                        name,
                                        image_bytes,
                                        pixels_per_grid,
                                    );
                                    builder = builder.append(tiled);
                                    has_content = true;
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to decode map image for tiled {}: {}",
                                        map.name, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 8. Token cutouts (paper standees for tabletop play)
    if opts.include_token_cutouts.unwrap_or(false) {
        info!("[SECTION] Token cutouts requested");
        use mimir_core::services::ModuleService;

        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules for token cutouts", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list modules for token cutouts: {}", e);
                vec![]
            }
        };

        let mut cutout_tokens: Vec<CutoutToken> = Vec::new();

        for module in modules {
            // Get module monsters
            let module_monsters = match dal::list_module_monsters(&mut db, &module.id) {
                Ok(monsters) => {
                    info!(
                        "  Module '{}' has {} monsters for cutouts",
                        module.name,
                        monsters.len()
                    );
                    monsters
                }
                Err(e) => {
                    error!(
                        "  Failed to list module {} monsters for cutouts: {}",
                        module.id, e
                    );
                    continue;
                }
            };

            for mm in module_monsters {
                // Look up monster size from catalog or homebrew
                let size = resolve_monster_size(&mut db, &mm, &campaign_id);

                // Try to load token image from assets (catalog monsters only)
                let image_bytes = load_monster_token_image(&mm, &app_state.paths.assets_dir);

                // Use display name if set, otherwise monster name
                let display_name = mm
                    .display_name
                    .or(mm.monster_name.clone())
                    .unwrap_or_else(|| "Unknown Monster".to_string());

                let mut token = CutoutToken::new(display_name, size, "monster".to_string())
                    .with_quantity(mm.quantity as u32);

                if let Some(bytes) = image_bytes {
                    token = token.with_image(bytes);
                }

                cutout_tokens.push(token);
            }
        }

        if !cutout_tokens.is_empty() {
            info!(
                "  Adding TokenCutoutSection with {} tokens",
                cutout_tokens.len()
            );
            let section = TokenCutoutSection::new(cutout_tokens);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No tokens with images found");
        }
    } else {
        info!("[SECTION] Token cutouts NOT requested");
    }

    if !has_content {
        error!("No content found to export - has_content is false");
        return ApiResponse::err("No content found to export");
    }

    info!("=== Building Campaign PDF ===");
    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!("Campaign PDF generated successfully ({} bytes)", size_bytes);

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate campaign PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export module documents to PDF
pub fn export_module_documents(
    app_state: &PrintCtx,
    print_state: &PrintState,
    module_id: String,
    options: Option<ModuleExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("Exporting module {} to PDF", module_id);

    let opts = options.unwrap_or_default();

    // Log received options
    info!("=== Module Export Options ===");
    info!("  include_documents: {:?}", opts.include_documents);
    info!("  include_monsters: {:?}", opts.include_monsters);
    info!("  include_traps: {:?}", opts.include_traps);
    info!("  include_pois: {:?}", opts.include_pois);
    info!("  include_npcs: {:?}", opts.include_npcs);
    info!("  include_play_notes: {:?}", opts.include_play_notes);
    info!("  include_preview: {:?}", opts.include_preview);
    info!("  preview_grid: {:?}", opts.preview_grid);
    info!("  preview_los_walls: {:?}", opts.preview_los_walls);
    info!("  preview_positions: {:?}", opts.preview_positions);
    info!("  include_play: {:?}", opts.include_play);
    info!("  play_grid: {:?}", opts.play_grid);
    info!("  play_los_walls: {:?}", opts.play_los_walls);
    info!("  play_cutouts: {:?}", opts.play_cutouts);
    info!("==============================");

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the module
    use mimir_core::services::ModuleService;
    let module = match ModuleService::new(&mut db).get(&module_id) {
        Ok(Some(m)) => m,
        Ok(None) => return ApiResponse::err(format!("Module not found: {}", module_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get module: {}", e)),
    };

    // Build combined PDF
    let mut builder = DocumentBuilder::new(&module.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(true)
        .with_toc(true);

    let mut has_content = false;

    // 1. Module documents
    if opts.include_documents.unwrap_or(true) {
        info!("[SECTION] Module documents requested");
        let documents = match DocumentService::new(&mut db).list_for_module(&module_id) {
            Ok(docs) => {
                info!("  Found {} documents", docs.len());
                docs
            }
            Err(e) => {
                error!("  Failed to list documents: {}", e);
                vec![]
            }
        };

        for doc in documents {
            match doc_section(&mut db, &app_state.paths.app_dir, &doc.content) {
                Ok(section) => {
                    info!("  Adding document: {}", doc.title);
                    builder = builder.append(section.with_title(&doc.title));
                    has_content = true;
                }
                Err(e) => {
                    error!("  Failed to parse document {}: {}", doc.id, e);
                }
            }
        }
    } else {
        info!("[SECTION] Module documents NOT requested");
    }

    // 2. Monster stat blocks
    if opts.include_monsters.unwrap_or(true) {
        info!("[SECTION] Monster cards requested");
        let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
            Ok(monsters) => {
                info!("  Found {} module monsters", monsters.len());
                monsters
            }
            Err(e) => {
                error!("  Failed to list module monsters: {}", e);
                vec![]
            }
        };

        let mut monster_data: Vec<Value> = Vec::new();
        for mm in &module_monsters {
            if let Some(mut data) = resolve_module_monster_data(&mut db, mm, &module.campaign_id) {
                // Apply display name override if set
                if let Some(ref display_name) = mm.display_name {
                    if let Some(obj) = data.as_object_mut() {
                        obj.insert("name".to_string(), Value::String(display_name.clone()));
                    }
                }
                // Add quantity copies if > 1
                for _ in 0..mm.quantity {
                    monster_data.push(data.clone());
                }
            }
        }

        if !monster_data.is_empty() {
            info!(
                "  Adding MonsterCardSection with {} monsters",
                monster_data.len()
            );
            let section = MonsterCardSection::new(monster_data);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No monster data found");
        }
    } else {
        info!("[SECTION] Monster cards NOT requested");
    }

    // 2b. Trap cards (from map_traps table)
    if opts.include_traps.unwrap_or(false) {
        use mimir_print::sections::TrapCardSection;
        use std::collections::HashSet;

        info!("[SECTION] Trap cards requested");

        // Get all maps for this module
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => m,
            Err(e) => {
                error!("  Failed to list module maps for traps: {}", e);
                vec![]
            }
        };

        // Collect unique trap names from all maps
        let mut seen_traps: HashSet<String> = HashSet::new();
        let mut trap_data: Vec<Value> = Vec::new();

        for map in &maps {
            match dal::list_map_traps(&mut db, &map.id) {
                Ok(map_traps) => {
                    for trap in map_traps {
                        // Skip if we've already added this trap type
                        if seen_traps.contains(&trap.name) {
                            continue;
                        }
                        seen_traps.insert(trap.name.clone());

                        // Try to look up trap in catalog (default to DMG source)
                        let sources_to_try = ["DMG", "XGE", "TCE", "PHB"];
                        let mut found_catalog = false;

                        for source in sources_to_try {
                            if let Ok(Some(catalog_trap)) =
                                catalog_dal::get_trap_by_name(&mut db, &trap.name, source)
                            {
                                if let Ok(data) = catalog_trap.parse_data() {
                                    info!("  Found trap in catalog: {} ({})", trap.name, source);
                                    trap_data.push(data);
                                    found_catalog = true;
                                    break;
                                }
                            }
                        }

                        // If not in catalog, create a basic trap card from map trap data
                        if !found_catalog {
                            info!("  Creating custom trap card: {}", trap.name);
                            let custom_trap = serde_json::json!({
                                "name": trap.name,
                                "trapHazType": "TRAP",
                                "effect": [
                                    {
                                        "type": "entries",
                                        "name": "Trigger",
                                        "entries": [trap.trigger_description.as_deref().unwrap_or("Unknown trigger")]
                                    },
                                    {
                                        "type": "entries",
                                        "name": "Effect",
                                        "entries": [trap.effect_description.as_deref().unwrap_or("Unknown effect")]
                                    }
                                ],
                                "countermeasures": [trap.description.as_deref().unwrap_or("No countermeasures listed")],
                                "dc": trap.dc
                            });
                            trap_data.push(custom_trap);
                        }
                    }
                }
                Err(e) => {
                    error!("  Failed to list traps for map {}: {}", map.id, e);
                }
            }
        }

        if !trap_data.is_empty() {
            info!("  Adding TrapCardSection with {} traps", trap_data.len());
            let section = TrapCardSection::new(trap_data);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No traps found in module maps");
        }
    } else {
        info!("[SECTION] Trap cards NOT requested");
    }

    // 2c. Points of Interest (from map_pois table)
    if opts.include_pois.unwrap_or(false) {
        info!("[SECTION] POIs requested");

        // Get all maps for this module
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => m,
            Err(e) => {
                error!("  Failed to list module maps for POIs: {}", e);
                vec![]
            }
        };

        // Collect POIs from all maps, grouped by map
        let mut poi_content = String::new();
        let mut total_pois = 0;

        for map in &maps {
            match dal::list_map_pois(&mut db, &map.id) {
                Ok(map_pois) => {
                    if !map_pois.is_empty() {
                        poi_content.push_str(&format!("## {}\n\n", map.name));
                        for poi in &map_pois {
                            total_pois += 1;
                            poi_content.push_str(&format!(
                                "### {} ({},{})\n",
                                poi.name, poi.grid_x, poi.grid_y
                            ));
                            if let Some(ref desc) = poi.description {
                                poi_content.push_str(&format!("{}\n", desc));
                            }
                            poi_content.push('\n');
                        }
                    }
                }
                Err(e) => {
                    error!("  Failed to list POIs for map {}: {}", map.id, e);
                }
            }
        }

        if !poi_content.is_empty() {
            info!(
                "  Adding POI section with {} points of interest",
                total_pois
            );
            let section = MarkdownSection::from_content(&poi_content, Some("Points of Interest"));
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No POIs found in module maps");
        }
    } else {
        info!("[SECTION] POIs NOT requested");
    }

    // 3. Map previews
    if opts.include_preview.unwrap_or(true) {
        info!("[SECTION] Map previews requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => {
                info!("  Found {} maps", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list module maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                info!("  Adding map preview: {}", map.name);
                                let preview =
                                    MapPreview::from_rendered(map.name.clone(), image_bytes);
                                builder = builder.append(preview);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("  Failed to decode map image for {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Map previews NOT requested");
    }

    // 4. Tiled maps for play
    if opts.include_play.unwrap_or(false) {
        info!("[SECTION] Tiled maps requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => {
                info!("  Found {} maps for tiled export", m.len());
                m
            }
            Err(e) => {
                error!("  Failed to list module maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    let pixels_per_grid = uvtt_json
                        .get("resolution")
                        .and_then(|r| r.get("pixels_per_grid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(70) as i32;

                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                let tiled = TiledMapSection::from_rendered(
                                    map.name.clone(),
                                    image_bytes,
                                    pixels_per_grid,
                                );
                                info!("  Adding tiled map: {}", map.name);
                                builder = builder.append(tiled);
                                has_content = true;
                            }
                            Err(e) => {
                                error!(
                                    "  Failed to decode map image for tiled {}: {}",
                                    map.name, e
                                );
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Tiled maps NOT requested");
    }

    // 5. Token cutouts
    if opts.play_cutouts.unwrap_or(false) {
        info!("[SECTION] Token cutouts requested");
        let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
            Ok(monsters) => {
                info!("  Found {} monsters for cutouts", monsters.len());
                monsters
            }
            Err(e) => {
                error!("  Failed to list module monsters for cutouts: {}", e);
                vec![]
            }
        };

        let mut cutout_tokens: Vec<CutoutToken> = Vec::new();

        for mm in module_monsters {
            // Look up monster size from catalog or homebrew
            let size = resolve_monster_size(&mut db, &mm, &module.campaign_id);

            // Try to load token image (catalog monsters only)
            let image_bytes = load_monster_token_image(&mm, &app_state.paths.assets_dir);

            let display_name = mm
                .display_name
                .or(mm.monster_name.clone())
                .unwrap_or_else(|| "Unknown Monster".to_string());
            let mut token = CutoutToken::new(display_name, size, "monster".to_string())
                .with_quantity(mm.quantity as u32);

            if let Some(bytes) = image_bytes {
                token = token.with_image(bytes);
            }

            cutout_tokens.push(token);
        }

        if !cutout_tokens.is_empty() {
            info!(
                "  Adding TokenCutoutSection with {} tokens",
                cutout_tokens.len()
            );
            let section = TokenCutoutSection::new(cutout_tokens);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No tokens with images found");
        }
    } else {
        info!("[SECTION] Token cutouts NOT requested");
    }

    if !has_content {
        error!("No content selected or found to export - has_content is false");
        return ApiResponse::err("No content selected or found to export");
    }

    info!("=== Building Module PDF ===");
    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!("Module PDF generated successfully ({} bytes)", size_bytes);

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate module PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

#[cfg(test)]
mod statblock_tests {
    use super::*;

    #[test]
    fn stat_block_lines_from_the_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let url = dir.path().join("mimir.db").to_string_lossy().into_owned();
        let mut conn = mimir_core::db::init_database(&url).unwrap();
        mimir_core::seed::srd::seed_srd_catalog(&mut conn).unwrap();
        let goblin = statblock_typst(&mut conn, "creature", "Goblin", Some("MM")).unwrap();
        assert!(goblin.contains("#\"Scimitar\""), "{goblin}");
        // A wrong source falls back to the name.
        assert!(statblock_typst(&mut conn, "creature", "Goblin", Some("LMoP")).is_some());
        assert!(statblock_typst(&mut conn, "creature", "Nobody", Some("XX")).is_none());
        assert!(statblock_typst(&mut conn, "deck", "Tarokka", None).is_none());
        let section = doc_section(&mut conn, dir.path(), "{@creature Goblin|MM}\n").unwrap();
        let pdf = DocumentBuilder::new("T").append(section).to_pdf().unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }
}
