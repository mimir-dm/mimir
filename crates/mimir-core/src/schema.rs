// @generated automatically by Diesel CLI.

diesel::table! {
    catalog_sources (code) {
        code -> Text,
        name -> Text,
        enabled -> Integer,
        imported_at -> Text,
    }
}
