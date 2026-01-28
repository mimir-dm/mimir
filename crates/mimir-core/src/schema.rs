// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
    }
}

diesel::table! {
    backgrounds (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    campaign_assets (id) {
        id -> Text,
        campaign_id -> Nullable<Text>,
        module_id -> Nullable<Text>,
        filename -> Text,
        description -> Nullable<Text>,
        mime_type -> Text,
        blob_path -> Text,
        file_size -> Nullable<Integer>,
        uploaded_at -> Text,
    }
}

diesel::table! {
    campaign_sources (id) {
        id -> Text,
        campaign_id -> Text,
        source_code -> Text,
    }
}

diesel::table! {
    campaigns (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        archived_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    catalog_sources (code) {
        code -> Text,
        name -> Text,
        enabled -> Integer,
        imported_at -> Text,
    }
}

diesel::table! {
    catalog_tables (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
    }
}

diesel::table! {
    books (id) {
        id -> Nullable<Integer>,
        source -> Text,
        name -> Text,
        data -> Text,
        contents -> Nullable<Text>,
        cover_path -> Nullable<Text>,
    }
}

diesel::table! {
    character_classes (id) {
        id -> Text,
        character_id -> Text,
        class_name -> Text,
        class_source -> Text,
        level -> Integer,
        subclass_name -> Nullable<Text>,
        subclass_source -> Nullable<Text>,
        starting_class -> Integer,
    }
}

diesel::table! {
    character_feats (id) {
        id -> Text,
        character_id -> Text,
        feat_name -> Text,
        feat_source -> Text,
        source_type -> Text,
    }
}

diesel::table! {
    character_features (id) {
        id -> Text,
        character_id -> Text,
        feature_type -> Text,
        feature_name -> Text,
        feature_source -> Text,
        source_class -> Text,
    }
}

diesel::table! {
    character_inventory (id) {
        id -> Text,
        character_id -> Text,
        item_name -> Text,
        item_source -> Text,
        quantity -> Integer,
        equipped -> Integer,
        attuned -> Integer,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    character_proficiencies (id) {
        id -> Text,
        character_id -> Text,
        proficiency_type -> Text,
        name -> Text,
        expertise -> Integer,
    }
}

diesel::table! {
    character_sources (id) {
        id -> Text,
        character_id -> Text,
        source_code -> Text,
    }
}

diesel::table! {
    character_spells (id) {
        id -> Text,
        character_id -> Text,
        spell_name -> Text,
        spell_source -> Text,
        source_class -> Text,
        prepared -> Integer,
    }
}

diesel::table! {
    characters (id) {
        id -> Text,
        campaign_id -> Text,
        name -> Text,
        is_npc -> Integer,
        player_name -> Nullable<Text>,
        race_name -> Nullable<Text>,
        race_source -> Nullable<Text>,
        background_name -> Nullable<Text>,
        background_source -> Nullable<Text>,
        strength -> Integer,
        dexterity -> Integer,
        constitution -> Integer,
        intelligence -> Integer,
        wisdom -> Integer,
        charisma -> Integer,
        cp -> Integer,
        sp -> Integer,
        ep -> Integer,
        gp -> Integer,
        pp -> Integer,
        traits -> Nullable<Text>,
        ideals -> Nullable<Text>,
        bonds -> Nullable<Text>,
        flaws -> Nullable<Text>,
        role -> Nullable<Text>,
        location -> Nullable<Text>,
        faction -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    classes (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    class_features (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        class_name -> Text,
        class_source -> Text,
        level -> Integer,
        data -> Text,
    }
}

diesel::table! {
    subclass_features (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        class_name -> Text,
        class_source -> Text,
        subclass_name -> Text,
        subclass_source -> Text,
        level -> Integer,
        data -> Text,
    }
}

diesel::table! {
    conditions (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    cults (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
    }
}

diesel::table! {
    deities (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        pantheon -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    diseases (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    documents (id) {
        id -> Text,
        campaign_id -> Text,
        module_id -> Nullable<Text>,
        title -> Text,
        content -> Text,
        doc_type -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    feats (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    hazards (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    item_attunement_classes (id) {
        id -> Nullable<Integer>,
        item_id -> Integer,
        class_name -> Text,
    }
}

diesel::table! {
    items (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        item_type -> Nullable<Text>,
        rarity -> Nullable<Text>,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    languages (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        language_type -> Nullable<Text>,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    light_sources (id) {
        id -> Text,
        map_id -> Text,
        grid_x -> Integer,
        grid_y -> Integer,
        name -> Nullable<Text>,
        bright_radius -> Integer,
        dim_radius -> Integer,
        color -> Nullable<Text>,
        active -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    map_traps (id) {
        id -> Text,
        map_id -> Text,
        grid_x -> Integer,
        grid_y -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        trigger_description -> Nullable<Text>,
        effect_description -> Nullable<Text>,
        dc -> Nullable<Integer>,
        triggered -> Integer,
        visible -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    map_pois (id) {
        id -> Text,
        map_id -> Text,
        grid_x -> Integer,
        grid_y -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        icon -> Text,
        color -> Nullable<Text>,
        visible -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    maps (id) {
        id -> Text,
        campaign_id -> Text,
        module_id -> Nullable<Text>,
        name -> Text,
        description -> Nullable<Text>,
        sort_order -> Integer,
        uvtt_asset_id -> Text,
        lighting_mode -> Text,
        fog_enabled -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    fog_revealed_areas (id) {
        id -> Text,
        map_id -> Text,
        x -> Double,
        y -> Double,
        width -> Double,
        height -> Double,
        created_at -> Text,
    }
}

diesel::table! {
    module_monsters (id) {
        id -> Text,
        module_id -> Text,
        monster_name -> Text,
        monster_source -> Text,
        display_name -> Nullable<Text>,
        notes -> Nullable<Text>,
        quantity -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    module_npcs (id) {
        id -> Text,
        module_id -> Text,
        name -> Text,
        role -> Nullable<Text>,
        description -> Nullable<Text>,
        appearance -> Nullable<Text>,
        personality -> Nullable<Text>,
        motivation -> Nullable<Text>,
        secrets -> Nullable<Text>,
        stat_block -> Nullable<Text>,
        token_asset_id -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    modules (id) {
        id -> Text,
        campaign_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        module_number -> Integer,
        created_at -> Text,
        updated_at -> Text,
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
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    objects (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        object_type -> Nullable<Text>,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    optional_features (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        feature_type -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    psionics (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        psionic_type -> Nullable<Text>,
        psionic_order -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    races (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    rewards (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        reward_type -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    senses (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        data -> Text,
    }
}

diesel::table! {
    skills (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        ability -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    spell_classes (id) {
        id -> Nullable<Integer>,
        spell_id -> Integer,
        class_name -> Text,
        source -> Text,
    }
}

diesel::table! {
    spell_subclasses (id) {
        id -> Nullable<Integer>,
        spell_id -> Integer,
        subclass_name -> Text,
        class_name -> Text,
        source -> Text,
    }
}

diesel::table! {
    spells (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        level -> Integer,
        school -> Nullable<Text>,
        ritual -> Integer,
        concentration -> Integer,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    subclasses (id) {
        id -> Nullable<Integer>,
        name -> Text,
        class_name -> Text,
        source -> Text,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    token_placements (id) {
        id -> Text,
        map_id -> Text,
        module_monster_id -> Nullable<Text>,
        module_npc_id -> Nullable<Text>,
        grid_x -> Integer,
        grid_y -> Integer,
        label -> Nullable<Text>,
        faction_color -> Nullable<Text>,
        hidden -> Integer,
        vision_bright_ft -> Nullable<Integer>,
        vision_dim_ft -> Nullable<Integer>,
        vision_dark_ft -> Integer,
        light_radius_ft -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    traps (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        trap_tier -> Nullable<Text>,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::table! {
    variant_rules (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        rule_type -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    vehicles (id) {
        id -> Nullable<Integer>,
        name -> Text,
        source -> Text,
        vehicle_type -> Nullable<Text>,
        data -> Text,
        fluff -> Nullable<Text>,
    }
}

diesel::joinable!(actions -> catalog_sources (source));
diesel::joinable!(backgrounds -> catalog_sources (source));
diesel::joinable!(campaign_assets -> campaigns (campaign_id));
diesel::joinable!(campaign_assets -> modules (module_id));
diesel::joinable!(campaign_sources -> campaigns (campaign_id));
diesel::joinable!(campaign_sources -> catalog_sources (source_code));
diesel::joinable!(catalog_tables -> catalog_sources (source));
diesel::joinable!(books -> catalog_sources (source));
diesel::joinable!(character_classes -> characters (character_id));
diesel::joinable!(character_feats -> characters (character_id));
diesel::joinable!(character_features -> characters (character_id));
diesel::joinable!(character_inventory -> characters (character_id));
diesel::joinable!(character_proficiencies -> characters (character_id));
diesel::joinable!(character_sources -> characters (character_id));
diesel::joinable!(character_sources -> catalog_sources (source_code));
diesel::joinable!(character_spells -> characters (character_id));
diesel::joinable!(characters -> campaigns (campaign_id));
diesel::joinable!(classes -> catalog_sources (source));
diesel::joinable!(conditions -> catalog_sources (source));
diesel::joinable!(cults -> catalog_sources (source));
diesel::joinable!(deities -> catalog_sources (source));
diesel::joinable!(diseases -> catalog_sources (source));
diesel::joinable!(documents -> campaigns (campaign_id));
diesel::joinable!(documents -> modules (module_id));
diesel::joinable!(feats -> catalog_sources (source));
diesel::joinable!(fog_revealed_areas -> maps (map_id));
diesel::joinable!(hazards -> catalog_sources (source));
diesel::joinable!(item_attunement_classes -> items (item_id));
diesel::joinable!(items -> catalog_sources (source));
diesel::joinable!(languages -> catalog_sources (source));
diesel::joinable!(light_sources -> maps (map_id));
diesel::joinable!(map_traps -> maps (map_id));
diesel::joinable!(map_pois -> maps (map_id));
diesel::joinable!(maps -> campaign_assets (uvtt_asset_id));
diesel::joinable!(maps -> campaigns (campaign_id));
diesel::joinable!(maps -> modules (module_id));
diesel::joinable!(module_monsters -> modules (module_id));
diesel::joinable!(module_npcs -> campaign_assets (token_asset_id));
diesel::joinable!(module_npcs -> modules (module_id));
diesel::joinable!(modules -> campaigns (campaign_id));
diesel::joinable!(monsters -> catalog_sources (source));
diesel::joinable!(objects -> catalog_sources (source));
diesel::joinable!(optional_features -> catalog_sources (source));
diesel::joinable!(psionics -> catalog_sources (source));
diesel::joinable!(races -> catalog_sources (source));
diesel::joinable!(rewards -> catalog_sources (source));
diesel::joinable!(senses -> catalog_sources (source));
diesel::joinable!(skills -> catalog_sources (source));
diesel::joinable!(spell_classes -> catalog_sources (source));
diesel::joinable!(spell_classes -> spells (spell_id));
diesel::joinable!(spell_subclasses -> catalog_sources (source));
diesel::joinable!(spell_subclasses -> spells (spell_id));
diesel::joinable!(spells -> catalog_sources (source));
diesel::joinable!(subclasses -> catalog_sources (source));
diesel::joinable!(token_placements -> maps (map_id));
diesel::joinable!(token_placements -> module_monsters (module_monster_id));
diesel::joinable!(token_placements -> module_npcs (module_npc_id));
diesel::joinable!(traps -> catalog_sources (source));
diesel::joinable!(variant_rules -> catalog_sources (source));
diesel::joinable!(vehicles -> catalog_sources (source));

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    backgrounds,
    books,
    campaign_assets,
    campaign_sources,
    campaigns,
    catalog_sources,
    catalog_tables,
    character_classes,
    character_feats,
    character_features,
    character_inventory,
    character_proficiencies,
    character_sources,
    character_spells,
    characters,
    classes,
    conditions,
    cults,
    deities,
    diseases,
    documents,
    feats,
    fog_revealed_areas,
    hazards,
    item_attunement_classes,
    items,
    languages,
    light_sources,
    map_traps,
    map_pois,
    maps,
    module_monsters,
    module_npcs,
    modules,
    monsters,
    objects,
    optional_features,
    psionics,
    races,
    rewards,
    senses,
    skills,
    spell_classes,
    spell_subclasses,
    spells,
    subclasses,
    token_placements,
    traps,
    variant_rules,
    vehicles,
);
