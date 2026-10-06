//! Development database seeder.
//!
//! Seeds the database with "The Lost Mine of Phandelver" test data
//! for development and testing.
//!
//! **Prerequisites**: Import MM (Monster Manual) via the Library
//! before seeding for full monster data display.
//!
//! With the `fixtures` feature, `seed_ui_fixture` seeds the SRD catalog plus
//! this campaign into an empty database: the fixture that development
//! machines use in place of real campaign data.

mod dev;
#[cfg(feature = "fixtures")]
mod fixture;
#[cfg(feature = "fixtures")]
pub mod srd;

pub use dev::{clear_dev_seed_data, is_already_seeded, seed_dev_data, TEST_CAMPAIGN_NAME};
#[cfg(feature = "fixtures")]
pub use fixture::{seed_ui_fixture, UiFixtureReport, FIXTURE_CAMPAIGN_NAME};
