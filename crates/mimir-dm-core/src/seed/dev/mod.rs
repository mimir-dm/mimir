//! Development database seeder.
//!
//! Seeds the database with "The Lost Mine of Phandelver" test data
//! for development and UAT testing.
//!
//! **Prerequisites**: The demo campaign uses monsters from MM (Monster Manual),
//! items from DMG, and spells from PHB. Import these books via the Library
//! before seeding for full functionality.

mod campaign;
mod characters;
mod maps;
mod module_data;

use crate::connection::DbConnection;
use crate::dal::campaign::campaigns::CampaignRepository;
use crate::dal::campaign::module_items::ModuleItemRepository;
use crate::dal::campaign::module_monsters::ModuleMonsterRepository;
use crate::dal::campaign::module_npcs::ModuleNpcRepository;
use crate::dal::campaign::modules::ModuleRepository;
use crate::dal::character::CharacterRepository;
use crate::dal::player::PlayerRepository;
use crate::error::Result;
use tracing::info;

pub use campaign::TEST_CAMPAIGN_NAME;

/// Check if dev seed data already exists.
pub fn is_already_seeded(conn: &mut DbConnection) -> Result<bool> {
    let mut repo = CampaignRepository::new(conn);
    let campaigns = repo.list()?;
    Ok(campaigns.iter().any(|c| c.name == TEST_CAMPAIGN_NAME))
}

/// Seed development data into the database.
///
/// Creates a test campaign with modules, characters, maps, and tokens.
/// Stateful - only seeds if no dev data exists yet.
///
/// **Prerequisites**: Import PHB, MM, and DMG books via Library for full
/// monster/spell/item data. The demo references these standard books.
///
/// Returns true if seeding was performed, false if data already existed.
pub fn seed_dev_data(conn: &mut DbConnection, campaigns_dir: &str, data_dir: &str) -> Result<bool> {
    if is_already_seeded(conn)? {
        info!("Dev seed data already exists, skipping");
        return Ok(false);
    }

    info!("Seeding development data...");

    // Check for required books and warn if missing
    check_required_books(conn);

    // 1. Players first
    let players = characters::seed_players(conn)?;
    info!("Created {} players", players.len());

    // 2. Concept character (no campaign needed)
    let concept = characters::seed_concept_character(conn, &players)?;
    info!("Created concept character: {}", concept.character_name);

    // 3. Campaign
    let campaign = campaign::seed_campaign(conn, campaigns_dir)?;
    info!("Created campaign: {}", campaign.name);

    // 4. Campaign characters (PCs and NPCs)
    let characters = characters::seed_characters(conn, campaign.id, &campaign.directory_path, &players)?;
    info!("Created {} campaign characters", characters.len());

    // 5. Modules (created in "planning" stage)
    let modules = campaign::seed_modules(conn, campaign.id)?;
    info!("Created {} modules", modules.len());

    // 6. Module data (monsters, NPCs, items)
    module_data::seed_monsters(conn, &modules)?;
    module_data::seed_npcs(conn, &modules, &characters)?;
    module_data::seed_items(conn, &modules)?;

    // 7. Maps and tokens
    maps::seed(conn, campaign.id, &modules, &characters, data_dir)?;

    info!("Dev seed data created successfully");
    Ok(true)
}

/// Check if required books are imported and log warnings if missing.
fn check_required_books(conn: &mut DbConnection) {
    use crate::schema::uploaded_books;
    use diesel::prelude::*;

    let required = ["PHB", "MM", "DMG"];
    let existing: Vec<String> = uploaded_books::table
        .select(uploaded_books::id)
        .load(conn)
        .unwrap_or_default();

    for book in required {
        if !existing.iter().any(|b| b == book) {
            tracing::warn!(
                "Required book '{}' not imported. Import via Library for full demo data.",
                book
            );
        }
    }
}

/// Clear existing dev seed data.
pub fn clear_dev_seed_data(conn: &mut DbConnection) -> Result<()> {
    use crate::dal::campaign::documents::DocumentRepository;

    let campaign_info = {
        let mut repo = CampaignRepository::new(conn);
        repo.list()?
            .into_iter()
            .find(|c| c.name == TEST_CAMPAIGN_NAME)
            .map(|c| (c.id, c.directory_path.clone()))
    };

    let Some((campaign_id, directory_path)) = campaign_info else {
        return Ok(());
    };

    info!("Clearing dev seed data for campaign {}", campaign_id);

    // Get module IDs
    let module_ids: Vec<i32> = {
        let mut repo = ModuleRepository::new(conn);
        repo.list_by_campaign(campaign_id)?
            .into_iter()
            .map(|m| m.id)
            .collect()
    };

    // Delete module associations
    for mid in &module_ids {
        ModuleMonsterRepository::new(conn).delete_by_module(*mid)?;
        ModuleNpcRepository::new(conn).delete_by_module(*mid)?;
        ModuleItemRepository::new(conn).delete_by_module(*mid)?;
    }

    // Delete documents
    for doc in DocumentRepository::find_by_campaign(conn, campaign_id)? {
        DocumentRepository::delete(conn, doc.id)?;
    }

    // Delete characters (campaign + concept)
    {
        let mut repo = CharacterRepository::new(conn);
        for c in repo.list_for_campaign(campaign_id)? {
            repo.delete(c.id)?;
        }
        // Delete concept character (no campaign) by name
        for c in repo.list_all()? {
            if c.campaign_id.is_none() && c.character_name == "Zephyr Stormborn" {
                repo.delete(c.id)?;
            }
        }
    }

    // Delete modules
    for mid in module_ids {
        ModuleRepository::new(conn).delete(mid)?;
    }

    // Delete dev players
    {
        let dev_players = ["Alice", "Bob", "Charlie", "Diana"];
        let mut repo = PlayerRepository::new(conn);
        for p in repo.list()? {
            if dev_players.contains(&p.name.as_str()) {
                repo.delete(p.id)?;
            }
        }
    }

    // Delete campaign
    CampaignRepository::new(conn).delete(campaign_id)?;

    // Delete campaign directory
    if std::path::Path::new(&directory_path).exists() {
        let _ = std::fs::remove_dir_all(&directory_path);
    }

    info!("Cleared dev seed data");
    Ok(())
}
