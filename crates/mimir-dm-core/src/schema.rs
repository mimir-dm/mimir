// @generated automatically by Diesel CLI.

// Schema is auto-generated - documentation would be overwritten
#![allow(missing_docs)]

diesel::table! {
    catalog_cults (id) {
        id -> Integer,
        name -> Text,
        category -> Text,
        cult_type -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_cult_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}
diesel::table! {
    campaigns (id) {
        id -> Integer,
        name -> Text,
        status -> Text,
        directory_path -> Text,
        created_at -> Text,
        session_zero_date -> Nullable<Text>,
        first_session_date -> Nullable<Text>,
        last_activity_at -> Text,
        archived_at -> Nullable<Text>,
    }
}

diesel::table! {
    modules (id) {
        id -> Integer,
        campaign_id -> Integer,
        name -> Text,
        module_number -> Integer,
        status -> Text,
        expected_sessions -> Integer,
        actual_sessions -> Integer,
        created_at -> Text,
        started_at -> Nullable<Text>,
        completed_at -> Nullable<Text>,
    }
}

diesel::table! {
    module_monsters (id) {
        id -> Integer,
        module_id -> Integer,
        monster_name -> Text,
        monster_source -> Text,
        quantity -> Integer,
        encounter_tag -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    sessions (id) {
        id -> Integer,
        campaign_id -> Integer,
        module_id -> Nullable<Integer>,
        session_number -> Integer,
        status -> Text,
        scheduled_date -> Nullable<Text>,
        prep_started_at -> Nullable<Text>,
        prep_completed_at -> Nullable<Text>,
        completed_at -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    workflow_cards (id) {
        id -> Text,
        board_type -> Text,
        title -> Text,
        description -> Nullable<Text>,
        created_at -> Text,
        last_moved_at -> Text,
        workflow_state -> Text,
        campaign_id -> Nullable<Integer>,
        module_id -> Nullable<Integer>,
        session_id -> Nullable<Integer>,
        priority -> Integer,
    }
}

diesel::table! {
    workflow_card_tags (card_id, tag) {
        card_id -> Text,
        tag -> Text,
    }
}

diesel::table! {
    template_documents (document_id, version_number) {
        document_id -> Text,
        version_number -> Integer,
        document_content -> Text,
        content_hash -> Text,
        document_type -> Nullable<Text>,
        document_level -> Nullable<Text>,
        purpose -> Nullable<Text>,
        variables_schema -> Nullable<Text>,
        default_values -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        is_active -> Bool,
        metadata -> Nullable<Text>,
    }
}

diesel::table! {
    documents (id) {
        id -> Integer,
        campaign_id -> Integer,
        module_id -> Nullable<Integer>,
        session_id -> Nullable<Integer>,
        template_id -> Text,
        document_type -> Text,
        title -> Text,
        file_path -> Text,
        created_at -> Text,
        updated_at -> Text,
        completed_at -> Nullable<Text>,
    }
}

diesel::table! {
    players (id) {
        id -> Integer,
        name -> Text,
        email -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    campaign_players (id) {
        id -> Integer,
        campaign_id -> Integer,
        player_id -> Integer,
        joined_at -> Text,
        active -> Bool,
    }
}

diesel::table! {
    characters (id) {
        id -> Integer,
        campaign_id -> Nullable<Integer>,
        player_id -> Nullable<Integer>,
        character_name -> Text,
        is_npc -> Bool,
        current_level -> Integer,
        current_version -> Integer,
        directory_path -> Text,
        created_at -> Text,
        updated_at -> Text,
        class -> Nullable<Text>,
        race -> Nullable<Text>,
    }
}

diesel::table! {
    character_versions (id) {
        id -> Integer,
        character_id -> Integer,
        version_number -> Integer,
        file_path -> Text,
        character_data -> Text,
        snapshot_reason -> Nullable<Text>,
        level -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    catalog_actions (id) {
        id -> Integer,
        name -> Text,
        time_type -> Text,
        description -> Text,
        see_also -> Nullable<Text>,
        source -> Text,
        full_action_json -> Text,
    }
}

diesel::table! {
    catalog_backgrounds (id) {
        id -> Integer,
        name -> Text,
        skills -> Text,
        languages -> Text,
        tools -> Text,
        feature -> Text,
        source -> Text,
        full_background_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_classes (id) {
        id -> Integer,
        name -> Text,
        hit_dice -> Nullable<Text>,
        primary_ability -> Nullable<Text>,
        proficiency -> Nullable<Text>,
        spellcasting_ability -> Nullable<Text>,
        subclass_title -> Nullable<Text>,
        caster_progression -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_class_json -> Text,
        fluff_json -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_subclasses (id) {
        id -> Integer,
        name -> Text,
        short_name -> Nullable<Text>,
        class_name -> Text,
        class_source -> Text,
        source -> Text,
        page -> Nullable<Integer>,
        caster_progression -> Nullable<Text>,
        spellcasting_ability -> Nullable<Text>,
        full_subclass_json -> Text,
        fluff_json -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_class_features (id) {
        id -> Integer,
        name -> Text,
        class_name -> Text,
        class_source -> Text,
        level -> Integer,
        source -> Text,
        page -> Nullable<Integer>,
        full_feature_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_subclass_features (id) {
        id -> Integer,
        name -> Text,
        class_name -> Text,
        class_source -> Text,
        subclass_short_name -> Nullable<Text>,
        subclass_source -> Text,
        level -> Integer,
        source -> Text,
        page -> Nullable<Integer>,
        full_feature_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_conditions (id) {
        id -> Integer,
        name -> Text,
        item_type -> Text,
        description -> Text,
        source -> Text,
        full_condition_json -> Text,
    }
}

diesel::table! {
    catalog_spells (id) {
        id -> Integer,
        name -> Text,
        level -> Integer,
        school -> Text,
        cast_time -> Text,
        range -> Text,
        components -> Text,
        tags -> Text,
        source -> Text,
        full_spell_json -> Text,
    }
}

diesel::table! {
    catalog_sources (source_name) {
        source_name -> Text,
        catalog_type -> Text,
        last_imported -> Nullable<Text>,
        file_path -> Text,
        file_hash -> Text,
        record_count -> Integer,
    }
}

diesel::table! {
    uploaded_books (id) {
        id -> Text,
        name -> Text,
        location -> Text,
        archive_path -> Text,
        uploaded_at -> Text,
        metadata_json -> Nullable<Text>,
    }
}

diesel::table! {
    catalog_languages (id) {
        id -> Integer,
        name -> Text,
        language_type -> Text,
        script -> Text,
        typical_speakers -> Text,
        source -> Text,
        full_language_json -> Text,
    }
}

diesel::table! {
    catalog_rewards (id) {
        id -> Integer,
        name -> Text,
        reward_type -> Text,
        description -> Text,
        has_prerequisites -> Integer,
        source -> Text,
        full_reward_json -> Text,
    }
}

diesel::table! {
    catalog_feats (id) {
        id -> Integer,
        name -> Text,
        prerequisites -> Nullable<Text>,
        brief -> Nullable<Text>,
        source -> Text,
        full_feat_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_races (id) {
        id -> Integer,
        name -> Text,
        size -> Nullable<Text>,
        speed -> Nullable<Integer>,
        ability_bonuses -> Nullable<Text>,
        traits_count -> Integer,
        source -> Text,
        full_race_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_objects (id) {
        id -> Integer,
        name -> Text,
        object_type -> Nullable<Text>,
        size -> Nullable<Text>,
        ac -> Nullable<Text>,
        hp -> Nullable<Text>,
        source -> Text,
        full_object_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_psionics (id) {
        id -> Integer,
        name -> Text,
        psionic_type -> Text,
        psionic_order -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_psionic_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_traps (id) {
        id -> Integer,
        name -> Text,
        category -> Text,
        trap_type -> Nullable<Text>,
        source -> Text,
        full_trap_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_items (id) {
        id -> Integer,
        name -> Text,
        item_type -> Nullable<Text>,
        type_name -> Nullable<Text>,
        rarity -> Nullable<Text>,
        value -> Nullable<Double>,
        weight -> Nullable<Double>,
        ac -> Nullable<Integer>,
        damage -> Nullable<Text>,
        requires_attunement -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_item_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_deities (id) {
        id -> Integer,
        name -> Text,
        title -> Nullable<Text>,
        pantheon -> Nullable<Text>,
        alignment -> Nullable<Text>,
        domains -> Nullable<Text>,
        symbol -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_deity_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_monsters (id) {
        id -> Integer,
        name -> Text,
        size -> Nullable<Text>,
        creature_type -> Nullable<Text>,
        alignment -> Nullable<Text>,
        cr -> Nullable<Text>,
        cr_numeric -> Nullable<Double>,
        hp -> Nullable<Integer>,
        ac -> Nullable<Integer>,
        source -> Text,
        page -> Nullable<Integer>,
        full_monster_json -> Text,
        fluff_json -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        token_image_path -> Nullable<Text>,
    }
}

diesel::table! {
    catalog_optional_features (id) {
        id -> Integer,
        name -> Text,
        feature_types -> Nullable<Text>,
        feature_type_full -> Nullable<Text>,
        prerequisite_text -> Nullable<Text>,
        grants_spells -> Nullable<Bool>,
        source -> Text,
        page -> Nullable<Integer>,
        full_optional_feature_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_variant_rules (id) {
        id -> Integer,
        name -> Text,
        rule_type -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_variant_rule_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_tables (id) {
        id -> Integer,
        name -> Text,
        caption -> Nullable<Text>,
        category -> Text,
        source -> Text,
        page -> Nullable<Integer>,
        columns_count -> Integer,
        rows_count -> Integer,
        full_table_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    catalog_vehicles (id) {
        id -> Integer,
        name -> Text,
        vehicle_type -> Nullable<Text>,
        size -> Nullable<Text>,
        cap_crew -> Nullable<Integer>,
        cap_passenger -> Nullable<Integer>,
        pace -> Nullable<Integer>,
        speed_text -> Nullable<Text>,
        terrain_text -> Nullable<Text>,
        source -> Text,
        page -> Nullable<Integer>,
        full_vehicle_json -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    maps (id) {
        id -> Integer,
        campaign_id -> Integer,
        module_id -> Nullable<Integer>,
        name -> Text,
        image_path -> Text,
        width_px -> Integer,
        height_px -> Integer,
        original_width_px -> Nullable<Integer>,
        original_height_px -> Nullable<Integer>,
        grid_type -> Text,
        grid_size_px -> Nullable<Integer>,
        grid_offset_x -> Integer,
        grid_offset_y -> Integer,
        created_at -> Text,
        updated_at -> Text,
        preview_path -> Nullable<Text>,
        fog_enabled -> Bool,
        ambient_light -> Text,
    }
}

diesel::table! {
    tokens (id) {
        id -> Integer,
        map_id -> Integer,
        name -> Text,
        token_type -> Text,
        size -> Text,
        x -> Float,
        y -> Float,
        visible_to_players -> Bool,
        color -> Nullable<Text>,
        image_path -> Nullable<Text>,
        monster_id -> Nullable<Integer>,
        character_id -> Nullable<Integer>,
        notes -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        vision_type -> Text,
        vision_range_ft -> Nullable<Float>,
    }
}

diesel::table! {
    fog_revealed_areas (id) {
        id -> Integer,
        map_id -> Integer,
        x -> Float,
        y -> Float,
        width -> Float,
        height -> Float,
        created_at -> Text,
    }
}

diesel::table! {
    light_sources (id) {
        id -> Integer,
        map_id -> Integer,
        token_id -> Nullable<Integer>,
        name -> Text,
        light_type -> Text,
        x -> Float,
        y -> Float,
        bright_radius_ft -> Float,
        dim_radius_ft -> Float,
        color -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    module_npcs (id) {
        id -> Integer,
        module_id -> Integer,
        character_id -> Integer,
        role -> Nullable<Text>,
        encounter_tag -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    module_items (id) {
        id -> Integer,
        module_id -> Integer,
        location -> Nullable<Text>,
        name -> Text,
        source -> Text,
        quantity -> Integer,
        notes -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(maps -> campaigns (campaign_id));
diesel::joinable!(modules -> campaigns (campaign_id));
diesel::joinable!(module_monsters -> modules (module_id));
diesel::joinable!(module_npcs -> modules (module_id));
diesel::joinable!(module_npcs -> characters (character_id));
diesel::joinable!(module_items -> modules (module_id));
diesel::joinable!(tokens -> maps (map_id));
diesel::joinable!(tokens -> catalog_monsters (monster_id));
diesel::joinable!(tokens -> characters (character_id));
diesel::joinable!(fog_revealed_areas -> maps (map_id));
diesel::joinable!(light_sources -> maps (map_id));
diesel::joinable!(light_sources -> tokens (token_id));
diesel::joinable!(sessions -> campaigns (campaign_id));
diesel::joinable!(sessions -> modules (module_id));
diesel::joinable!(workflow_cards -> campaigns (campaign_id));
diesel::joinable!(workflow_cards -> modules (module_id));
diesel::joinable!(workflow_cards -> sessions (session_id));
diesel::joinable!(workflow_card_tags -> workflow_cards (card_id));
diesel::joinable!(documents -> campaigns (campaign_id));
diesel::joinable!(documents -> modules (module_id));
diesel::joinable!(documents -> sessions (session_id));
diesel::joinable!(campaign_players -> campaigns (campaign_id));
diesel::joinable!(campaign_players -> players (player_id));
diesel::joinable!(characters -> campaigns (campaign_id));
diesel::joinable!(characters -> players (player_id));
diesel::joinable!(character_versions -> characters (character_id));

diesel::allow_tables_to_appear_in_same_query!(
    campaigns,
    campaign_players,
    maps,
    modules,
    module_monsters,
    module_npcs,
    module_items,
    sessions,
    tokens,
    fog_revealed_areas,
    light_sources,
    workflow_cards,
    workflow_card_tags,
    template_documents,
    documents,
    players,
    characters,
    character_versions,
    catalog_actions,
    catalog_backgrounds,
    catalog_classes,
    catalog_subclasses,
    catalog_class_features,
    catalog_subclass_features,
    catalog_conditions,
    catalog_cults,
    catalog_deities,
    catalog_feats,
    catalog_items,
    catalog_languages,
    catalog_monsters,
    catalog_objects,
    catalog_optional_features,
    catalog_psionics,
    catalog_races,
    catalog_rewards,
    catalog_spells,
    catalog_sources,
    catalog_tables,
    catalog_traps,
    catalog_variant_rules,
    catalog_vehicles,
    uploaded_books,
);
