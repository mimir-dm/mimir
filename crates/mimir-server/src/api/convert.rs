//! `mimir-core` models to `mimir-wire` types. The conversions live here so
//! that mimir-wire stays free of mimir-core (and builds for wasm).

use mimir_core::models::campaign as core;
use mimir_wire as wire;

pub fn campaign(c: core::Campaign) -> wire::CampaignSummary {
    wire::CampaignSummary {
        id: c.id,
        name: c.name,
        description: c.description,
        archived_at: c.archived_at,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

pub fn module(m: core::Module) -> wire::ModuleSummary {
    wire::ModuleSummary {
        id: m.id,
        campaign_id: m.campaign_id,
        name: m.name,
        description: m.description,
        module_number: m.module_number,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}

pub fn document_summary(d: core::Document) -> wire::DocumentSummary {
    wire::DocumentSummary {
        id: d.id,
        campaign_id: d.campaign_id,
        module_id: d.module_id,
        title: d.title,
        doc_type: d.doc_type,
        sort_order: d.sort_order,
        updated_at: d.updated_at,
    }
}

pub fn document(d: core::Document) -> wire::Document {
    wire::Document {
        id: d.id,
        campaign_id: d.campaign_id,
        module_id: d.module_id,
        title: d.title,
        doc_type: d.doc_type,
        sort_order: d.sort_order,
        content: d.content,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

pub fn character(c: core::CharacterResponse) -> wire::CharacterSummary {
    let classes: Vec<wire::ClassLevel> = c
        .classes
        .into_iter()
        .map(|k| wire::ClassLevel {
            class_name: k.class_name,
            subclass_name: k.subclass_name,
            level: k.level,
        })
        .collect();
    wire::CharacterSummary {
        id: c.id,
        campaign_id: c.campaign_id,
        name: c.name,
        is_npc: c.is_npc != 0,
        player_name: c.player_name,
        race_name: c.race_name,
        background_name: c.background_name,
        level: classes.iter().map(|k| k.level).sum(),
        classes,
        role: c.role,
        location: c.location,
        faction: c.faction,
        updated_at: c.updated_at,
    }
}

pub fn module_npc(n: core::ModuleNpc) -> wire::ModuleNpcSummary {
    wire::ModuleNpcSummary {
        id: n.id,
        module_id: n.module_id,
        name: n.name,
        role: n.role,
        description: n.description,
    }
}

/// `homebrew_name`: the name of the homebrew monster, when it is one.
pub fn module_monster(
    m: core::ModuleMonster,
    homebrew_name: Option<String>,
) -> wire::ModuleMonsterSummary {
    let name = m
        .display_name
        .clone()
        .or_else(|| m.monster_name.clone())
        .or(homebrew_name)
        .unwrap_or_else(|| "Unnamed monster".to_string());
    wire::ModuleMonsterSummary {
        id: m.id,
        module_id: m.module_id,
        name,
        monster_name: m.monster_name,
        monster_source: m.monster_source,
        homebrew_monster_id: m.homebrew_monster_id,
        quantity: m.quantity,
        notes: m.notes,
    }
}

pub fn map(m: core::Map) -> wire::MapSummary {
    wire::MapSummary {
        id: m.id,
        campaign_id: m.campaign_id,
        module_id: m.module_id,
        name: m.name,
        description: m.description,
        sort_order: m.sort_order,
        lighting_mode: m.lighting_mode,
        fog_enabled: m.fog_enabled != 0,
    }
}
