//! Reward Data Access Layer
//!
//! Database operations for rewards.

use crate::models::catalog::{NewReward, Reward};
use crate::schema::rewards;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new reward.
pub fn insert_reward(conn: &mut SqliteConnection, reward: &NewReward) -> QueryResult<i32> {
    diesel::insert_into(rewards::table)
        .values(reward)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple rewards in a batch.
pub fn insert_rewards(conn: &mut SqliteConnection, rewards: &[NewReward]) -> QueryResult<usize> {
    diesel::insert_into(rewards::table)
        .values(rewards)
        .execute(conn)
}

/// Get a reward by its ID.
pub fn get_reward(conn: &mut SqliteConnection, id: i32) -> QueryResult<Reward> {
    rewards::table.filter(rewards::id.eq(id)).first(conn)
}

/// Get a reward by name and source.
pub fn get_reward_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Reward>> {
    rewards::table
        .filter(rewards::name.eq(name))
        .filter(rewards::source.eq(source))
        .first(conn)
        .optional()
}

/// List all rewards, ordered by name.
pub fn list_rewards(conn: &mut SqliteConnection) -> QueryResult<Vec<Reward>> {
    rewards::table.order(rewards::name.asc()).load(conn)
}

/// List rewards from a specific source.
pub fn list_rewards_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Reward>> {
    rewards::table
        .filter(rewards::source.eq(source))
        .order(rewards::name.asc())
        .load(conn)
}

/// List rewards by type (e.g., "blessing", "boon", "charm").
pub fn list_rewards_by_type(
    conn: &mut SqliteConnection,
    reward_type: &str,
) -> QueryResult<Vec<Reward>> {
    rewards::table
        .filter(rewards::reward_type.eq(reward_type))
        .order(rewards::name.asc())
        .load(conn)
}

/// Delete a reward by its ID.
pub fn delete_reward(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(rewards::table.filter(rewards::id.eq(id))).execute(conn)
}

/// Delete all rewards from a specific source.
pub fn delete_rewards_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(rewards::table.filter(rewards::source.eq(source))).execute(conn)
}

/// Count all rewards.
pub fn count_rewards(conn: &mut SqliteConnection) -> QueryResult<i64> {
    rewards::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_source;
    use crate::models::catalog::NewCatalogSource;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            "CREATE TABLE catalog_sources (
                code TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                imported_at TEXT NOT NULL
            );
            CREATE TABLE rewards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
                reward_type TEXT,
                data TEXT NOT NULL,
                UNIQUE(name, source)
            );",
        )
        .expect("Failed to create tables");

        let source = NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert source");

        conn
    }

    #[test]
    fn test_reward_crud() {
        let mut conn = setup_test_db();

        let reward = NewReward::new("Blessing of Health", "DMG", r#"{"name":"Blessing of Health"}"#)
            .with_type("blessing");
        let id = insert_reward(&mut conn, &reward).expect("Failed to insert");

        let retrieved = get_reward(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Blessing of Health");
        assert_eq!(retrieved.reward_type, Some("blessing".to_string()));

        delete_reward(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_rewards(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_by_type() {
        let mut conn = setup_test_db();

        let rewards_data = vec![
            NewReward::new("Blessing of Health", "DMG", r#"{}"#).with_type("blessing"),
            NewReward::new("Blessing of Protection", "DMG", r#"{}"#).with_type("blessing"),
            NewReward::new("Boon of Immortality", "DMG", r#"{}"#).with_type("boon"),
        ];
        insert_rewards(&mut conn, &rewards_data).expect("Failed to insert");

        let blessings = list_rewards_by_type(&mut conn, "blessing").expect("Failed to list");
        assert_eq!(blessings.len(), 2);

        let boons = list_rewards_by_type(&mut conn, "boon").expect("Failed to list");
        assert_eq!(boons.len(), 1);
    }
}
