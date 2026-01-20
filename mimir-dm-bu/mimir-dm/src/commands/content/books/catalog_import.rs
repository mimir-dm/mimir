//! Catalog import functionality for books.
//!
//! Provides functions to import all catalog data types from uploaded book archives.
//! Handles spells, items, monsters, and 17 other entity types from 5e Tools format.

use diesel::prelude::*;
use mimir_dm_core::services::{
    ActionService, BackgroundService, ClassService, ConditionService, CultService, DeityService,
    FeatService, ItemService, LanguageService, MonsterService, ObjectService,
    OptionalFeatureService, PsionicService, RaceService, RewardService, SpellService, TableService,
    TrapService, VariantRuleService, VehicleService,
};
use std::path::Path;
use tracing::{info, warn};

/// Import all catalog content from a book
///
/// This function imports all 20 catalog types from a book directory:
/// - Spells
/// - Actions
/// - Conditions
/// - Languages
/// - Rewards
/// - Backgrounds
/// - Feats
/// - Races
/// - Objects
/// - Traps/Hazards
/// - Cults/Boons
/// - Variant Rules
/// - Optional Features
/// - Items
/// - Monsters
/// - Deities
/// - Vehicles
/// - Classes
/// - Psionics
/// - Tables
///
/// Each import is attempted independently. If an import fails, a warning is logged
/// but the function continues to import the remaining catalog types.
///
/// # Arguments
///
/// * `conn` - Mutable reference to the database connection
/// * `book_dir` - Path to the directory containing the book data
/// * `book_id` - The unique identifier for the book
pub fn import_all_catalogs_from_book(conn: &mut SqliteConnection, book_dir: &Path, book_id: &str) {
    // Import spells
    match SpellService::import_spells_from_book(conn, book_dir, book_id) {
        Ok(spell_count) => {
            info!("Imported {} spells from book '{}'", spell_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import spells: {}",
                e
            );
        }
    }

    // Import actions
    match ActionService::import_actions_from_book(conn, book_dir, book_id) {
        Ok(action_count) => {
            info!("Imported {} actions from book '{}'", action_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import actions: {}",
                e
            );
        }
    }

    // Import conditions
    match ConditionService::import_conditions_from_book(conn, book_dir, book_id) {
        Ok(condition_count) => {
            info!(
                "Imported {} conditions from book '{}'",
                condition_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import conditions: {}",
                e
            );
        }
    }

    // Import languages
    match LanguageService::import_languages_from_book(conn, book_dir, book_id) {
        Ok(language_count) => {
            info!(
                "Imported {} languages from book '{}'",
                language_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import languages: {}",
                e
            );
        }
    }

    // Import rewards
    match RewardService::import_rewards_from_book(conn, book_dir, book_id) {
        Ok(reward_count) => {
            info!("Imported {} rewards from book '{}'", reward_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import rewards: {}",
                e
            );
        }
    }

    // Import backgrounds
    match BackgroundService::import_backgrounds_from_book(conn, book_dir, book_id) {
        Ok(background_count) => {
            info!(
                "Imported {} backgrounds from book '{}'",
                background_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import backgrounds: {}",
                e
            );
        }
    }

    // Import feats
    match FeatService::import_feats_from_book(conn, book_dir, book_id) {
        Ok(feat_count) => {
            info!("Imported {} feats from book '{}'", feat_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import feats: {}",
                e
            );
        }
    }

    // Import races
    match RaceService::import_races_from_book(conn, book_dir, book_id) {
        Ok(race_count) => {
            info!("Imported {} races from book '{}'", race_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import races: {}",
                e
            );
        }
    }

    // Import objects
    match ObjectService::import_objects_from_book(conn, book_dir, book_id) {
        Ok(object_count) => {
            info!("Imported {} objects from book '{}'", object_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import objects: {}",
                e
            );
        }
    }

    // Import traps and hazards
    match TrapService::import_traps_from_book(conn, book_dir, book_id) {
        Ok(trap_count) => {
            info!(
                "Imported {} traps/hazards from book '{}'",
                trap_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import traps/hazards: {}",
                e
            );
        }
    }

    // Import cults and boons
    match CultService::import_cults_from_book(conn, book_dir, book_id) {
        Ok(cult_count) => {
            info!(
                "Imported {} cults/boons from book '{}'",
                cult_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import cults/boons: {}",
                e
            );
        }
    }

    // Import variant rules
    match VariantRuleService::import_variant_rules_from_book(conn, book_dir, book_id) {
        Ok(variant_rule_count) => {
            info!(
                "Imported {} variant rules from book '{}'",
                variant_rule_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import variant rules: {}",
                e
            );
        }
    }

    // Import optional features
    match OptionalFeatureService::import_optional_features_from_book(conn, book_dir, book_id) {
        Ok(optional_feature_count) => {
            info!(
                "Imported {} optional features from book '{}'",
                optional_feature_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import optional features: {}",
                e
            );
        }
    }

    // Import items
    match ItemService::import_items_from_book(conn, book_dir, book_id) {
        Ok(item_count) => {
            info!("Imported {} items from book '{}'", item_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import items: {}",
                e
            );
        }
    }

    // Import monsters
    match MonsterService::import_monsters_from_book(conn, book_dir, book_id) {
        Ok(monster_count) => {
            info!(
                "Imported {} monsters from book '{}'",
                monster_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import monsters: {}",
                e
            );
        }
    }

    // Import deities
    match DeityService::import_deities_from_book(conn, book_dir, book_id) {
        Ok(deity_count) => {
            info!("Imported {} deities from book '{}'", deity_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import deities: {}",
                e
            );
        }
    }

    // Import vehicles
    match VehicleService::import_vehicles_from_book(conn, book_dir, book_id) {
        Ok(vehicle_count) => {
            info!(
                "Imported {} vehicles from book '{}'",
                vehicle_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import vehicles: {}",
                e
            );
        }
    }

    // Import classes
    match ClassService::import_classes_from_book(conn, book_dir, book_id) {
        Ok(class_count) => {
            info!("Imported {} classes from book '{}'", class_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import classes: {}",
                e
            );
        }
    }

    // Import psionics
    match PsionicService::import_psionics_from_book(conn, book_dir, book_id) {
        Ok(psionic_count) => {
            info!(
                "Imported {} psionics from book '{}'",
                psionic_count, book_id
            );
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import psionics: {}",
                e
            );
        }
    }

    // Import tables
    match TableService::import_tables_from_book(conn, book_dir, book_id) {
        Ok(table_count) => {
            info!("Imported {} tables from book '{}'", table_count, book_id);
        }
        Err(e) => {
            warn!(
                "Book uploaded successfully but failed to import tables: {}",
                e
            );
        }
    }
}
