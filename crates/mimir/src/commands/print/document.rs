//! Document Export Commands
//!
//! Tauri commands for exporting campaign and module documents to PDF.

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
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

use super::helpers::{compute_ac, compute_hit_die_string, compute_hp_max, enrich_inventory_item};
use super::{ApiResponse, CampaignExportOptions, ModuleExportOptions, PrintResult, PrintTemplateInfo};

/// List available print templates
#[tauri::command]
pub fn list_print_templates(
    print_state: State<'_, PrintState>,
) -> ApiResponse<Vec<PrintTemplateInfo>> {
    use mimir_print::PrintService;

    let service = PrintService::new(print_state.templates_dir.clone());
    match service.list_templates() {
        Ok(templates) => {
            let infos: Vec<PrintTemplateInfo> = templates
                .into_iter()
                .map(|t| PrintTemplateInfo {
                    id: t.id,
                    name: t.name,
                    category: t.category,
                })
                .collect();
            ApiResponse::ok(infos)
        }
        Err(e) => ApiResponse::err(format!("Failed to list templates: {}", e)),
    }
}

/// Export a single campaign document to PDF
#[tauri::command]
pub fn export_campaign_document(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
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
    let section = match MarkdownSection::from_markdown(&document.content) {
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

/// Export all campaign documents to PDF
#[tauri::command]
pub fn export_campaign_documents(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    campaign_id: String,
    options: Option<CampaignExportOptions>,
) -> ApiResponse<PrintResult> {
    use mimir_core::services::ModuleService;

    info!("Exporting campaign {} to PDF", campaign_id);

    let opts = options.unwrap_or_default();

    // Log received options
    info!("=== Campaign Export Options ===");
    info!("  include_campaign_docs: {:?}", opts.include_campaign_docs);
    info!("  include_module_content: {:?}", opts.include_module_content);
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
            match MarkdownSection::from_markdown(&doc.content) {
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
                match MarkdownSection::from_markdown(&doc.content) {
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
                    if let Ok(Some(catalog_monster)) =
                        catalog_dal::get_monster_by_name(&mut db, &mm.monster_name, &mm.monster_source)
                    {
                        if let Ok(mut data) = catalog_monster.parse_data() {
                            if let Some(ref display_name) = mm.display_name {
                                if let Some(obj) = data.as_object_mut() {
                                    obj.insert(
                                        "name".to_string(),
                                        Value::String(display_name.clone()),
                                    );
                                }
                            }
                            for _ in 0..mm.quantity {
                                monster_data.push(data.clone());
                            }
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
                                    "**{}** ({})\n",
                                    poi.name,
                                    format!("{},{}", poi.grid_x, poi.grid_y)
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
                            level: c.level,
                            subclass_name: c.subclass_name,
                            is_starting,
                        }
                    })
                    .collect(),
                inventory: inventory
                    .iter()
                    .map(|i| enrich_inventory_item(&mut db, i))
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
            let mut char_data = char_data;
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
                // Look up catalog monster to get size
                let size = match catalog_dal::get_monster_by_name(
                    &mut db,
                    &mm.monster_name,
                    &mm.monster_source,
                ) {
                    Ok(Some(catalog_monster)) => {
                        catalog_monster.size.unwrap_or_else(|| "Medium".to_string())
                    }
                    _ => "Medium".to_string(),
                };

                // Try to load token image from assets
                let img_base = app_state
                    .paths
                    .assets_dir
                    .join("catalog")
                    .join("bestiary")
                    .join("tokens")
                    .join(&mm.monster_source);

                let extensions = ["webp", "png", "jpg", "jpeg"];
                let mut image_bytes: Option<Vec<u8>> = None;

                for ext in &extensions {
                    let path = img_base.join(format!("{}.{}", &mm.monster_name, ext));
                    if path.exists() {
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                image_bytes = Some(bytes);
                                break;
                            }
                            Err(e) => {
                                error!("Failed to read token image {:?}: {}", path, e);
                            }
                        }
                    }
                }

                // Use display name if set, otherwise monster name
                let display_name = mm.display_name.unwrap_or(mm.monster_name.clone());

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
#[tauri::command]
pub fn export_module_documents(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
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
            match MarkdownSection::from_markdown(&doc.content) {
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
            if let Ok(Some(catalog_monster)) =
                catalog_dal::get_monster_by_name(&mut db, &mm.monster_name, &mm.monster_source)
            {
                if let Ok(mut data) = catalog_monster.parse_data() {
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
                                "### {} ({})\n",
                                poi.name,
                                format!("{},{}", poi.grid_x, poi.grid_y)
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
            let size = match catalog_dal::get_monster_by_name(
                &mut db,
                &mm.monster_name,
                &mm.monster_source,
            ) {
                Ok(Some(catalog_monster)) => {
                    catalog_monster.size.unwrap_or_else(|| "Medium".to_string())
                }
                _ => "Medium".to_string(),
            };

            // Try to load token image
            let img_base = app_state
                .paths
                .assets_dir
                .join("catalog")
                .join("bestiary")
                .join("tokens")
                .join(&mm.monster_source);

            let extensions = ["webp", "png", "jpg", "jpeg"];
            let mut image_bytes: Option<Vec<u8>> = None;

            for ext in &extensions {
                let path = img_base.join(format!("{}.{}", &mm.monster_name, ext));
                if path.exists() {
                    if let Ok(bytes) = std::fs::read(&path) {
                        image_bytes = Some(bytes);
                        break;
                    }
                }
            }

            let display_name = mm.display_name.unwrap_or(mm.monster_name.clone());
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
