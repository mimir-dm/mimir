//! The REST API under `/api/v1` (COLLIERY-I-0612).
//!
//! Every route is in [`routes`] with the role it needs. The router puts the
//! DM routes behind the DM bearer auth; player routes join with player links
//! (MIMIR-T-0716). Handlers call the `mimir-core` services, never the DAL,
//! and answer with `mimir-wire` types ([`convert`]).

pub mod campaigns;
pub mod combat;
pub mod convert;
pub mod display;
pub mod links;
pub mod maps;
pub mod scope;

use axum::middleware;
use axum::routing::{delete, get, patch, post, MethodRouter};
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
    use campaigns as c;
    use maps as m;
    use Access::{Dm, Player};
    const G: &[&str] = &["GET"];
    vec![
        route("/session", Player, G, get(crate::session)),
        // Player links (DM).
        route(
            "/characters/{id}/link",
            Dm,
            &["GET", "POST", "DELETE"],
            get(links::status).post(links::issue).delete(links::revoke),
        ),
        // Campaign content (read).
        route("/campaigns", Dm, G, get(c::list_campaigns)),
        route("/campaigns/{id}", Dm, G, get(c::get_campaign)),
        route(
            "/campaigns/{id}/documents",
            Dm,
            G,
            get(c::campaign_documents),
        ),
        route("/campaigns/{id}/modules", Dm, G, get(c::campaign_modules)),
        route("/campaigns/{id}/pcs", Dm, G, get(c::campaign_pcs)),
        route("/campaigns/{id}/npcs", Dm, G, get(c::campaign_npcs)),
        route("/campaigns/{id}/maps", Dm, G, get(c::campaign_maps)),
        route(
            "/campaigns/{id}/display",
            Player,
            &["GET", "PUT"],
            get(display::get_display).put(display::set_display),
        ),
        route("/modules/{id}", Dm, G, get(c::get_module)),
        route("/modules/{id}/documents", Dm, G, get(c::module_documents)),
        route("/modules/{id}/monsters", Dm, G, get(c::module_monsters)),
        route("/modules/{id}/npcs", Dm, G, get(c::module_npcs)),
        route("/modules/{id}/maps", Dm, G, get(c::module_maps)),
        route("/documents/{id}", Dm, G, get(c::get_document)),
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
        route("/maps/{id}", Dm, G, get(m::get_map)),
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
