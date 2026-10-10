//! The REST API under `/api/v1` (COLLIERY-I-0612).
//!
//! Every route is in [`routes`] with the role it needs. The router puts the
//! DM routes behind the DM bearer auth; player routes join with player links
//! (MIMIR-T-0716). Handlers call the `mimir-core` services, never the DAL,
//! and answer with `mimir-wire` types ([`convert`]).

pub mod campaigns;
pub mod convert;
pub mod display;
pub mod maps;

use axum::middleware;
use axum::routing::{delete, get, patch, post, MethodRouter};
use axum::Router;

use crate::auth;
use crate::state::AppState;

/// Who may call a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// The DM token (or open mode).
    Dm,
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
    use Access::Dm;
    const G: &[&str] = &["GET"];
    vec![
        route("/session", Dm, G, get(crate::session)),
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
            Dm,
            &["GET", "PUT"],
            get(display::get_display).put(display::set_display),
        ),
        route("/modules/{id}", Dm, G, get(c::get_module)),
        route("/modules/{id}/documents", Dm, G, get(c::module_documents)),
        route("/modules/{id}/monsters", Dm, G, get(c::module_monsters)),
        route("/modules/{id}/npcs", Dm, G, get(c::module_npcs)),
        route("/modules/{id}/maps", Dm, G, get(c::module_maps)),
        route("/documents/{id}", Dm, G, get(c::get_document)),
        // Maps.
        route("/maps/{id}", Dm, G, get(m::get_map)),
        route("/maps/{id}/geometry", Dm, G, get(m::get_geometry)),
        route("/maps/{id}/image", Dm, G, get(m::get_map_image)),
        route("/maps/{id}/player-view", Dm, G, get(m::player_view)),
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
        route("/tokens/{id}/image", Dm, G, get(m::get_token_image)),
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
    for r in routes() {
        match r.access {
            Access::Dm => dm = dm.route(r.path, r.handler),
        }
    }
    dm.route_layer(middleware::from_fn_with_state(state, auth::require_dm))
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
