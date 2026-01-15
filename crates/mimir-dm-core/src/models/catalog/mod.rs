//! D&D 5e catalog models for game content
//!
//! This module contains all the data structures for static game content
//! like spells, items, monsters, classes, and more.
//!
//! # Module Organization
//!
//! - **Entity modules** (`monster.rs`, `class.rs`, etc.) - Hand-maintained types
//!   with database mapping and custom serialization
//! - **`types`** - Shared polymorphic types used across entities
//! - **`schema`** - Schema management and download utilities
//! - **`generated`** - Reference types generated from JSON Schema (feature-gated)

pub mod types;
pub mod schema;

#[cfg(feature = "generated-types")]
pub mod generated;
pub mod action;
pub mod background;
pub mod book;
pub mod class;
pub mod condition;
pub mod cult;
pub mod deity;
pub mod feat;
pub mod item;
pub mod language;
pub mod monster;
pub mod object;
pub mod optionalfeature;
pub mod psionic;
pub mod race;
pub mod reward;
pub mod spell;
pub mod table;
pub mod trap;
pub mod variant_rule;
pub mod vehicle;

// Re-export commonly used types
pub use class::{
    Class, ClassData, ClassFeature, ClassFeatureData, ClassFluff, ClassFluffData, ClassSummary,
    HitDice, Multiclassing, MulticlassingProficiencies, StartingEquipment, StartingProficiencies,
    Subclass, SubclassFeature, SubclassFluff,
};

pub use item::{Item, ItemData, ItemSummary};

pub use monster::{
    ArmorClass, CreatureType, HitPoints, Monster, MonsterData, MonsterFluff, MonsterFluffData,
    MonsterSummary, Saves, Skills, Speed,
};

pub use spell::{
    CastingTime, CatalogSpell, ClassReference, Classes, Components, Distance, Duration,
    DurationValue, MaterialComponent, NewCatalogSpell, ScalingLevelDice, Spell, SpellData,
    SpellFilters, SpellMeta, SpellRange, SpellSchool, SpellSummary, SubclassReference,
    SubclassReference2,
};

pub use feat::{CatalogFeat, Feat, FeatData, FeatFilters, FeatSummary, NewCatalogFeat};

pub use race::{
    CatalogRace, NewCatalogRace, Race, RaceData, RaceFilters, RaceFluff, RaceFluffData,
    RaceSummary, Subrace,
};

pub use background::{
    Background, BackgroundData, BackgroundFilters, BackgroundFluff, BackgroundFluffData,
    BackgroundSummary, BackgroundWithDetails, CatalogBackground, NewCatalogBackground,
};

pub use book::{NewUploadedBook, UploadedBook};

pub use action::{
    Action, ActionFilters, ActionSummary, ActionTime, CatalogAction, NewCatalogAction,
};

pub use condition::{
    CatalogCondition, Condition, ConditionData, ConditionFilters, ConditionFluff,
    ConditionFluffData, ConditionOrDisease, ConditionSummary, ConditionWithDetails, Disease,
    DiseaseData, NewCatalogCondition,
};

pub use optionalfeature::{OptionalFeature, OptionalFeatureData, OptionalFeatureSummary};

pub use deity::{Deity, DeityData, DeitySummary};

pub use object::{
    CatalogObject, DndObject, NewCatalogObject, ObjectData, ObjectFilters, ObjectSummary,
};

pub use trap::{
    CatalogTrap, Hazard, HazardData, NewCatalogTrap, Trap, TrapData, TrapFilters, TrapOrHazard,
    TrapSummary,
};

pub use language::{
    CatalogLanguage, Language, LanguageData, LanguageFilters, LanguageFluff, LanguageFluffData,
    LanguageSummary, NewCatalogLanguage,
};

pub use reward::{
    CatalogReward, NewCatalogReward, Reward, RewardData, RewardFilters, RewardFluff,
    RewardFluffData, RewardSummary,
};

pub use table::{
    CatalogTable, NewCatalogTable, Table, TableData, TableFilters, TableFluff, TableFluffData,
    TableSummary,
};

pub use variant_rule::{VariantRule, VariantRuleData, VariantRuleSummary};

pub use vehicle::{Speed as VehicleSpeed, Vehicle, VehicleData, VehicleSummary, VehicleWeapon};

pub use cult::{Boon, BoonData, Cult, CultBoonSummary, CultData};

pub use psionic::{
    CatalogPsionic, ConcentrationDuration, NewCatalogPsionic, Psionic, PsionicCost, PsionicFilters,
    PsionicMode, PsionicSummary,
};

// Shared types for 5etools data
pub use types::{
    // Entry system
    Entry, EntryObject, Image, ImageHref, RollSpec, TableCell, TableCellObject,
    // Proficiency/choice types
    AbilityBonus, AbilityChoice, ChooseSpec, ProficiencyChoice, ProficiencyItem,
    // Monster types
    AlignmentComponent, AlignmentValue, ArmorClassEntry, ArmorClassValue, ChallengeRatingValue,
    CreatureTag, CreatureTypeValue, HitPointsValue, SpeedValue,
    // Damage modifier types
    DamageModifier, DamageModifierChoice,
    // Prerequisite types
    Prerequisite, PrerequisiteClass, PrerequisiteLevel, PrerequisiteObject, PrerequisiteOther,
    PrerequisiteRace,
    // Spell types
    AdditionalSpellEntry, AdditionalSpellList, AdditionalSpells, SpellcastingAbility,
    ConsumeValue,
    // Starting Equipment types
    StartingEquipmentEntry, StartingEquipmentChoiceGroup, StartingEquipmentItem,
    StartingEquipmentItemObject,
    // Source types
    OtherSource,
};
