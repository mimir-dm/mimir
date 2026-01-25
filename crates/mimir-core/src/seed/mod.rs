//! Development database seeder.
//!
//! Seeds the database with "The Lost Mine of Phandelver" test data
//! for development and testing.
//!
//! **Prerequisites**: Import MM (Monster Manual) via the Library
//! before seeding for full monster data display.

mod dev;

pub use dev::{clear_dev_seed_data, is_already_seeded, seed_dev_data, TEST_CAMPAIGN_NAME};
