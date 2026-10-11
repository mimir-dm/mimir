//! The REST API under `/api/v1` (COLLIERY-I-0612).
//!
//! Every route is in [`routes`] with the role it needs. The router puts the
//! DM routes behind the DM bearer auth; player routes join with player links
//! (MIMIR-T-0716). Handlers call the `mimir-core` services, never the DAL,
//! and answer with `mimir-wire` types ([`convert`]).

pub mod admin;
pub mod assets;
pub mod authoring;
pub mod campaigns;
pub mod catalog;
pub mod characters;
pub mod combat;
pub mod convert;
pub mod display;
pub mod homebrew;
pub mod links;
pub mod maps;
pub mod party;
pub mod print;
pub mod reference;
pub mod scope;
pub mod sheet;

use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{delete, get, patch, post, put, MethodRouter};
use axum::Router;

use crate::auth;
use crate::state::AppState;

/// Who may call a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// The DM only (the DM token, or open mode). A player gets 403.
    Dm,
    /// The DM and players; the handler limits a player to their scope
    /// (their campaign, the shown map, their character).
    Player,
}

/// One path of the API, with its methods.
pub struct ApiRoute {
    /// The methods, for the docs and the tests (the handler is the truth).
    pub methods: &'static [&'static str],
    /// The path under `/api/v1`.
    pub path: &'static str,
    pub access: Access,
    handler: MethodRouter<AppState>,
}

fn route(
    path: &'static str,
    access: Access,
    methods: &'static [&'static str],
    handler: MethodRouter<AppState>,
) -> ApiRoute {
    ApiRoute {
        methods,
        path,
        access,
        handler,
    }
}

/// The route table.
pub fn routes() -> Vec<ApiRoute> {
    use admin as ad;
    use authoring as w;
    use campaigns as c;
    use catalog as cat;
    use characters as ch;
    use homebrew as hb;
    use maps as m;
    use print as pdf;
    use reference as rf;
    use Access::{Dm, Player};
    const G: &[&str] = &["GET"];
    vec![
        route("/session", Player, G, get(crate::session)),
        // Characters (MIMIR-T-0717): a player reaches their own.
        route(
            "/characters/{id}",
            Player,
            &["GET", "PATCH", "DELETE"],
            get(ch::get).patch(ch::patch).delete(ch::delete),
        ),
        route(
            "/characters/{id}/inventory",
            Player,
            &["POST"],
            post(ch::add_item),
        ),
        route(
            "/inventory/{id}",
            Player,
            &["PATCH", "DELETE"],
            patch(ch::patch_item).delete(ch::remove_item),
        ),
        route(
            "/characters/{id}/spells",
            Player,
            &["POST"],
            post(ch::add_spell),
        ),
        route(
            "/characters/{id}/spells/{spell_id}",
            Player,
            &["PATCH", "DELETE"],
            patch(ch::patch_spell).delete(ch::remove_spell),
        ),
        route(
            "/characters/{id}/level-up",
            Player,
            &["POST"],
            post(ch::level_up),
        ),
        // Notes and the party (MIMIR-T-0718).
        route(
            "/characters/{id}/notes",
            Player,
            &["GET", "POST"],
            get(party::list).post(party::add),
        ),
        route(
            "/notes/{id}",
            Player,
            &["PATCH", "DELETE"],
            patch(party::patch).delete(party::delete),
        ),
        route("/modules/{id}/notes", Dm, G, get(party::module_notes)),
        route(
            "/characters/{id}/sharing",
            Player,
            &["PUT"],
            put(party::set_sharing),
        ),
        route("/campaigns/{id}/party", Player, G, get(party::party)),
        // Catalog choices (reference data).
        route("/catalog/classes", Player, G, get(cat::classes)),
        route(
            "/catalog/classes/{name}/{source}/level-info",
            Player,
            G,
            get(cat::class_level_info),
        ),
        route(
            "/catalog/classes/{name}/subclasses",
            Player,
            G,
            get(cat::subclasses),
        ),
        route("/catalog/feats", Player, G, get(cat::feats)),
        route(
            "/catalog/optional-features",
            Player,
            G,
            get(cat::optional_features),
        ),
        route("/catalog/spells", Player, G, get(cat::spells)),
        route("/catalog/items", Player, G, get(cat::items)),
        route("/catalog/monsters", Dm, G, get(cat::monsters)),
        // The catalog browser and the book reader (MIMIR-T-0723).
        route("/catalog/search/{category}", Dm, G, get(rf::search)),
        route(
            "/catalog/entry/{category}/{source}/{name}",
            Dm,
            G,
            get(rf::entry),
        ),
        route("/catalog/sources", Dm, G, get(rf::sources)),
        // Settings and admin (MIMIR-T-0724).
        route(
            "/catalog/sources/{code}",
            Dm,
            &["DELETE"],
            delete(ad::delete_source),
        ),
        route(
            "/catalog/import",
            Dm,
            &["POST"],
            post(ad::import_catalog).layer(DefaultBodyLimit::max(ad::CATALOG_LIMIT)),
        ),
        route(
            "/catalog/import-images",
            Dm,
            &["POST"],
            post(ad::import_images).layer(DefaultBodyLimit::max(ad::CATALOG_LIMIT)),
        ),
        route("/jobs/{id}", Dm, G, get(ad::job)),
        // PDF export (MIMIR-T-0724).
        route("/characters/{id}/pdf", Dm, G, get(pdf::character)),
        // The text a sheet names (MIMIR-T-0733).
        route("/characters/{id}/features", Player, G, get(sheet::features)),
        route(
            "/characters/{id}/entry/{category}/{source}/{name}",
            Player,
            G,
            get(sheet::entry),
        ),
        route("/campaigns/{id}/pdf", Dm, G, get(pdf::campaign)),
        route("/modules/{id}/pdf", Dm, G, get(pdf::module)),
        route("/maps/{id}/pdf", Dm, G, get(pdf::map)),
        route("/documents/{id}/pdf", Dm, G, get(pdf::document)),
        route(
            "/modules/{id}/monster-cards",
            Dm,
            G,
            get(pdf::monster_cards),
        ),
        route("/campaigns/{id}/export", Dm, G, get(ad::export_archive)),
        route(
            "/archives/preview",
            Dm,
            &["POST"],
            post(ad::preview_archive).layer(DefaultBodyLimit::max(ad::ARCHIVE_LIMIT)),
        ),
        route(
            "/archives/import",
            Dm,
            &["POST"],
            post(ad::import_archive).layer(DefaultBodyLimit::max(ad::ARCHIVE_LIMIT)),
        ),
        route(
            "/campaigns/{id}/sources",
            Dm,
            &["GET", "PUT"],
            get(ad::campaign_sources).put(ad::set_campaign_sources),
        ),
        route(
            "/characters/{id}/sources",
            Dm,
            &["GET", "PUT"],
            get(ad::character_sources).put(ad::set_character_sources),
        ),
        // Homebrew (MIMIR-T-0722).
        route(
            "/campaigns/{id}/homebrew/{kind}",
            Dm,
            &["GET", "POST"],
            get(hb::list).post(hb::create),
        ),
        route(
            "/homebrew/{kind}/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(hb::get).patch(hb::patch).delete(hb::delete),
        ),
        route("/homebrew/{kind}/{id}/usage", Dm, G, get(hb::usage_of)),
        route("/books/{source}", Dm, G, get(rf::book)),
        route("/book-images/{*path}", Dm, G, get(rf::book_image)),
        route("/catalog/races", Dm, G, get(cat::races)),
        route("/catalog/backgrounds", Dm, G, get(cat::backgrounds)),
        route(
            "/catalog/creation-skills",
            Dm,
            G,
            get(cat::creation_skills_of),
        ),
        // The DM's character management (MIMIR-T-0721).
        route("/characters", Dm, &["POST"], post(ch::create)),
        route("/characters/unassigned", Dm, G, get(ch::unassigned)),
        route(
            "/characters/{id}/campaign",
            Dm,
            &["PUT"],
            put(ch::set_campaign),
        ),
        // Player links (DM).
        route(
            "/characters/{id}/link",
            Dm,
            &["GET", "POST", "DELETE"],
            get(links::status).post(links::issue).delete(links::revoke),
        ),
        // Campaign content (read).
        // Campaign content: read, and the DM's writes (MIMIR-T-0720).
        route(
            "/campaigns",
            Dm,
            &["GET", "POST"],
            get(c::list_campaigns).post(w::create_campaign),
        ),
        route(
            "/campaigns/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(c::get_campaign)
                .patch(w::patch_campaign)
                .delete(w::delete_campaign),
        ),
        route(
            "/campaigns/{id}/archive",
            Dm,
            &["PUT", "DELETE"],
            put(w::archive_campaign).delete(w::unarchive_campaign),
        ),
        route(
            "/campaigns/{id}/documents",
            Dm,
            &["GET", "POST"],
            get(c::campaign_documents).post(w::create_campaign_document),
        ),
        route(
            "/campaigns/{id}/modules",
            Dm,
            &["GET", "POST"],
            get(c::campaign_modules).post(w::create_module),
        ),
        route("/campaigns/{id}/pcs", Dm, G, get(c::campaign_pcs)),
        route("/campaigns/{id}/npcs", Dm, G, get(c::campaign_npcs)),
        route(
            "/campaigns/{id}/maps",
            Dm,
            &["GET", "POST"],
            get(c::campaign_maps)
                .post(w::create_map)
                .layer(DefaultBodyLimit::max(w::MAP_UPLOAD_LIMIT)),
        ),
        // Images (MIMIR-T-0731).
        route(
            "/campaigns/{id}/images",
            Dm,
            &["GET", "POST"],
            get(assets::campaign_images)
                .post(assets::upload_image)
                .layer(DefaultBodyLimit::max(assets::IMAGE_UPLOAD_LIMIT)),
        ),
        route(
            "/modules/{id}/images",
            Dm,
            &["GET", "POST"],
            get(assets::module_images)
                .post(assets::upload_module_image)
                .layer(DefaultBodyLimit::max(assets::IMAGE_UPLOAD_LIMIT)),
        ),
        route("/assets/{id}/file", Dm, G, get(assets::image_file)),
        route(
            "/assets/{id}",
            Dm,
            &["DELETE"],
            delete(assets::delete_image),
        ),
        route(
            "/campaigns/{id}/display",
            Player,
            &["GET", "PUT"],
            get(display::get_display).put(display::set_display),
        ),
        route(
            "/modules/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(c::get_module)
                .patch(w::patch_module)
                .delete(w::delete_module),
        ),
        route("/modules/{id}/move", Dm, &["POST"], post(w::move_module)),
        route(
            "/modules/{id}/documents",
            Dm,
            &["GET", "POST"],
            get(c::module_documents).post(w::create_module_document),
        ),
        route(
            "/modules/{id}/monsters",
            Dm,
            &["GET", "POST"],
            get(c::module_monsters).post(w::add_module_monster),
        ),
        route(
            "/module-monsters/{id}",
            Dm,
            &["PATCH", "DELETE"],
            patch(w::patch_module_monster).delete(w::delete_module_monster),
        ),
        route("/modules/{id}/npcs", Dm, G, get(c::module_npcs)),
        route("/modules/{id}/maps", Dm, G, get(c::module_maps)),
        route(
            "/documents/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(c::get_document)
                .patch(w::patch_document)
                .delete(w::delete_document),
        ),
        route(
            "/documents/{id}/move",
            Dm,
            &["POST"],
            post(w::move_document),
        ),
        // Combat.
        route(
            "/modules/{id}/combat",
            Dm,
            &["GET", "POST"],
            get(combat::active).post(combat::start),
        ),
        route(
            "/combat/{id}",
            Dm,
            &["GET", "DELETE"],
            get(combat::get).delete(combat::end),
        ),
        route("/combat/{id}/next", Dm, &["POST"], post(combat::next)),
        route(
            "/combat/{id}/previous",
            Dm,
            &["POST"],
            post(combat::previous),
        ),
        route("/combat/{id}/entries", Dm, &["POST"], post(combat::add)),
        route(
            "/combat/{id}/link-tokens",
            Dm,
            &["POST"],
            post(combat::link_tokens),
        ),
        route(
            "/combat-entries/{id}",
            Dm,
            &["PATCH", "DELETE"],
            patch(combat::patch_entry).delete(combat::remove_entry),
        ),
        route(
            "/combat-entries/{id}/damage",
            Dm,
            &["POST"],
            post(combat::damage),
        ),
        route(
            "/combat-entries/{id}/heal",
            Dm,
            &["POST"],
            post(combat::heal),
        ),
        route(
            "/combat-entries/{id}/conditions",
            Dm,
            &["POST"],
            post(combat::add_condition),
        ),
        route(
            "/combat-entries/{id}/conditions/{name}",
            Dm,
            &["DELETE"],
            delete(combat::remove_condition),
        ),
        // Maps.
        route(
            "/maps/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(m::get_map).patch(w::patch_map).delete(w::delete_map),
        ),
        route("/maps/{id}/geometry", Player, G, get(m::get_geometry)),
        route("/maps/{id}/image", Player, G, get(m::get_map_image)),
        route("/maps/{id}/player-view", Player, G, get(m::player_view)),
        route(
            "/maps/{id}/tokens",
            Dm,
            &["GET", "POST"],
            get(m::list_tokens).post(m::create_token),
        ),
        route(
            "/tokens/{id}",
            Dm,
            &["PATCH", "DELETE"],
            patch(m::update_token).delete(m::delete_token),
        ),
        route("/tokens/{id}/image", Player, G, get(m::get_token_image)),
        route(
            "/maps/{id}/fog",
            Dm,
            &["GET", "PUT"],
            get(m::get_fog).put(m::set_fog),
        ),
        route("/maps/{id}/fog/reveal", Dm, &["POST"], post(m::reveal)),
        route("/maps/{id}/doors", Dm, G, get(m::list_doors)),
        route("/maps/{id}/doors/{index}", Dm, &["PUT"], put(m::set_door)),
        route(
            "/maps/{id}/fog/revealed",
            Dm,
            &["DELETE"],
            delete(m::reset_fog),
        ),
        route(
            "/maps/{id}/fog/revealed/{area_id}",
            Dm,
            &["DELETE"],
            delete(m::delete_fog_area),
        ),
        route(
            "/maps/{id}/lights",
            Dm,
            &["GET", "POST", "DELETE"],
            get(m::list_lights)
                .post(m::create_light)
                .delete(m::delete_all_lights),
        ),
        route(
            "/lights/{id}",
            Dm,
            &["PATCH", "DELETE"],
            patch(m::update_light).delete(m::delete_light),
        ),
        route(
            "/maps/{id}/traps",
            Dm,
            &["GET", "POST"],
            get(m::list_traps).post(m::create_trap),
        ),
        route(
            "/traps/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(m::get_trap)
                .patch(m::update_trap)
                .delete(m::delete_trap),
        ),
        route(
            "/maps/{id}/pois",
            Dm,
            &["GET", "POST"],
            get(m::list_pois).post(m::create_poi),
        ),
        route(
            "/pois/{id}",
            Dm,
            &["GET", "PATCH", "DELETE"],
            get(m::get_poi).patch(m::update_poi).delete(m::delete_poi),
        ),
    ]
}

/// The `/api/v1` router: each route behind the auth of its role.
pub fn router(state: AppState) -> Router<AppState> {
    let mut dm = Router::new();
    let mut player = Router::new();
    for r in routes() {
        match r.access {
            Access::Dm => dm = dm.route(r.path, r.handler),
            Access::Player => player = player.route(r.path, r.handler),
        }
    }
    dm.route_layer(middleware::from_fn(auth::dm_only))
        .merge(player)
        .route_layer(middleware::from_fn_with_state(state, auth::authenticate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn each_route_is_listed_once() {
        let table = routes();
        let unique: HashSet<_> = table.iter().map(|r| r.path).collect();
        assert_eq!(unique.len(), table.len());
        assert!(table
            .iter()
            .all(|r| r.path.starts_with('/') && !r.methods.is_empty()));
    }
}
