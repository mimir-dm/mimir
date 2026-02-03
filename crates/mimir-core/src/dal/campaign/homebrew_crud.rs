//! Homebrew CRUD Macro
//!
//! Generates identical DAL functions for each homebrew entity type,
//! eliminating ~170 lines of copy-paste boilerplate.

/// Generate standard homebrew CRUD functions for a given entity type.
///
/// Produces: insert, get, get_by_name, list, update, delete, delete_all (by campaign).
macro_rules! homebrew_crud {
    (
        table: $table:ident,
        model: $model:ty,
        new_model: $new_model:ty,
        update_model: $update_model:ty,
        singular: $singular:ident,
        plural: $plural:ident
    ) => {
        paste::paste! {
            #[doc = "Insert a new homebrew " $singular "."]
            pub fn [<insert_campaign_homebrew_ $singular>](
                conn: &mut diesel::SqliteConnection,
                record: &$new_model,
            ) -> diesel::QueryResult<String> {
                use diesel::prelude::*;
                diesel::insert_into($table::table)
                    .values(record)
                    .execute(conn)?;
                Ok(record.id.to_string())
            }

            #[doc = "Get a homebrew " $singular " by ID."]
            pub fn [<get_campaign_homebrew_ $singular>](
                conn: &mut diesel::SqliteConnection,
                id: &str,
            ) -> diesel::QueryResult<$model> {
                use diesel::prelude::*;
                $table::table.find(id).first(conn)
            }

            #[doc = "Get a homebrew " $singular " by campaign_id and name."]
            pub fn [<get_campaign_homebrew_ $singular _by_name>](
                conn: &mut diesel::SqliteConnection,
                campaign_id: &str,
                name: &str,
            ) -> diesel::QueryResult<Option<$model>> {
                use diesel::prelude::*;
                $table::table
                    .filter($table::campaign_id.eq(campaign_id))
                    .filter($table::name.eq(name))
                    .first(conn)
                    .optional()
            }

            #[doc = "List all homebrew " $plural " for a campaign."]
            pub fn [<list_campaign_homebrew_ $plural>](
                conn: &mut diesel::SqliteConnection,
                campaign_id: &str,
            ) -> diesel::QueryResult<Vec<$model>> {
                use diesel::prelude::*;
                $table::table
                    .filter($table::campaign_id.eq(campaign_id))
                    .order($table::name.asc())
                    .load(conn)
            }

            #[doc = "Update a homebrew " $singular "."]
            pub fn [<update_campaign_homebrew_ $singular>](
                conn: &mut diesel::SqliteConnection,
                id: &str,
                update: &$update_model,
            ) -> diesel::QueryResult<usize> {
                use diesel::prelude::*;
                diesel::update($table::table.find(id))
                    .set(update)
                    .execute(conn)
            }

            #[doc = "Delete a homebrew " $singular " by ID."]
            pub fn [<delete_campaign_homebrew_ $singular>](
                conn: &mut diesel::SqliteConnection,
                id: &str,
            ) -> diesel::QueryResult<usize> {
                use diesel::prelude::*;
                diesel::delete($table::table.find(id)).execute(conn)
            }

            #[doc = "Delete all homebrew " $plural " for a campaign."]
            pub fn [<delete_campaign_homebrew_ $plural>](
                conn: &mut diesel::SqliteConnection,
                campaign_id: &str,
            ) -> diesel::QueryResult<usize> {
                use diesel::prelude::*;
                diesel::delete(
                    $table::table
                        .filter($table::campaign_id.eq(campaign_id)),
                )
                .execute(conn)
            }
        }
    };
}

pub(crate) use homebrew_crud;
