//! Module data seeding - monsters, NPCs, and items.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::modules::Module;
use crate::models::character::Character;
use crate::services::{ModuleItemService, ModuleMonsterService, ModuleNpcService};
use tracing::info;

/// Seed monsters for modules.
///
/// Uses monsters from MM (Monster Manual). Make sure MM is imported
/// via Library for monster details to display correctly.
pub fn seed_monsters(conn: &mut DbConnection, modules: &[Module]) -> Result<()> {
    let hideout = modules.iter().find(|m| m.name == "Cragmaw Hideout");

    if let Some(module) = hideout {
        // Monsters from Monster Manual (MM)
        let monsters: &[(&str, &str, i32, Option<&str>)] = &[
            ("Goblin", "MM", 6, Some("Cave Entrance")),
            ("Goblin", "MM", 3, Some("Guard Post")),
            ("Goblin", "MM", 5, Some("Main Chamber")),
            ("Bugbear", "MM", 1, Some("Main Chamber")), // Klarg
            ("Wolf", "MM", 2, Some("Kennel")),
            ("Goblin Boss", "MM", 1, Some("Boss Chamber")),
            ("Goblin", "MM", 2, Some("Boss Chamber")),
            ("Adult Black Dragon", "MM", 1, Some("Dragon Lair")),
        ];

        for (name, source, qty, tag) in monsters {
            let mut service = ModuleMonsterService::new(conn);
            service.add_monster(module.id, name.to_string(), source.to_string(), *qty, tag.map(String::from))?;
        }
    }
    info!("Seeded module monsters");
    Ok(())
}

/// Seed NPCs for modules.
pub fn seed_npcs(conn: &mut DbConnection, modules: &[Module], characters: &[Character]) -> Result<()> {
    let hideout = modules.iter().find(|m| m.name == "Cragmaw Hideout").map(|m| m.id);
    let find_npc = |name: &str| characters.iter().find(|c| c.character_name == name).map(|c| c.id);

    let sildar = find_npc("Sildar Hallwinter");
    let gundren = find_npc("Gundren Rockseeker");
    let glasstaff = find_npc("Iarno Albrek");

    let mut count = 0;

    if let Some(mid) = hideout {
        // Sildar - prisoner
        if let Some(cid) = sildar {
            let mut s = ModuleNpcService::new(conn);
            s.add_npc(mid, cid, Some("captive".into()), Some("boss_chamber".into()), Some("Prisoner of Klarg".into()))?;
            count += 1;
        }

        // Gundren - plot hook
        if let Some(cid) = gundren {
            let mut s = ModuleNpcService::new(conn);
            s.add_npc(mid, cid, Some("plot_hook".into()), None, Some("Taken to Cragmaw Castle".into()))?;
            count += 1;
        }

        // Glasstaff - antagonist
        if let Some(cid) = glasstaff {
            let mut s = ModuleNpcService::new(conn);
            s.add_npc(mid, cid, Some("antagonist".into()), None, Some("Working with Black Spider".into()))?;
            count += 1;
        }
    }

    info!("Linked {} NPCs to modules", count);
    Ok(())
}

/// Seed items for modules.
pub fn seed_items(conn: &mut DbConnection, modules: &[Module]) -> Result<()> {
    let hideout = modules.iter().find(|m| m.name == "Cragmaw Hideout").map(|m| m.id);

    if let Some(mid) = hideout {
        let mut s = ModuleItemService::new(conn);
        s.add_item(mid, "Potion of Healing".into(), "DMG".into(), 3, Some("store_room".into()), Some("On shelf".into()))?;
        s.add_item(mid, "Jade Frog Statuette".into(), "custom".into(), 1, Some("boss_chamber".into()), Some("Worth 40gp".into()))?;
        s.add_item(mid, "+1 Longsword".into(), "DMG".into(), 1, Some("boss_chamber".into()), None)?;
        s.add_item(mid, "Spell Scroll (2nd Level)".into(), "DMG".into(), 1, Some("hidden_cache".into()), Some("DC 15 to find".into()))?;
        info!("Added 4 items to module");
    }

    Ok(())
}
