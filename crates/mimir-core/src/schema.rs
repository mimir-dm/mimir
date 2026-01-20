// @generated automatically by Diesel CLI.

diesel::table! {
    catalog_sources (code) {
        code -> Text,
        name -> Text,
        enabled -> Integer,
        imported_at -> Text,
    }
}

diesel::table! {
    monsters (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        cr -> Nullable<Text>,
        creature_type -> Nullable<Text>,
        size -> Nullable<Text>,
        token_image_path -> Nullable<Text>,
        data -> Text,
    }
}

diesel::joinable!(monsters -> catalog_sources (source));

diesel::allow_tables_to_appear_in_same_query!(catalog_sources, monsters,);
