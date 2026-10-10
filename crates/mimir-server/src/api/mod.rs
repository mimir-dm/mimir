//! The REST API under `/api/v1` (COLLIERY-I-0612).
//!
//! Every route is in [`routes`] with the role it needs. The router puts the
//! DM routes behind the DM bearer auth; player routes join with player links
//! (MIMIR-T-0716). Handlers call the `mimir-core` services, never the DAL,
//! and answer with `mimir-wire` types ([`convert`]).

pub mod campaigns;
pub mod convert;

use axum::middleware;
use axum::routing::{get, MethodRouter};
use axum::Router;

use crate::auth;
use crate::state::AppState;

/// Who may call a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// The DM token (or open mode).
    Dm,
}

/// One route of the API.
pub struct ApiRoute {
    pub method: &'static str,
    /// The path under `/api/v1`.
    pub path: &'static str,
    pub access: Access,
    handler: MethodRouter<AppState>,
}

fn route(path: &'static str, access: Access, handler: MethodRouter<AppState>) -> ApiRoute {
    ApiRoute {
        method: "GET",
        path,
        access,
        handler,
    }
}

/// The route table.
pub fn routes() -> Vec<ApiRoute> {
    use campaigns as c;
    use Access::Dm;
    vec![
        route("/session", Dm, get(crate::session)),
        route("/campaigns", Dm, get(c::list_campaigns)),
        route("/campaigns/{id}", Dm, get(c::get_campaign)),
        route("/campaigns/{id}/documents", Dm, get(c::campaign_documents)),
        route("/campaigns/{id}/modules", Dm, get(c::campaign_modules)),
        route("/campaigns/{id}/pcs", Dm, get(c::campaign_pcs)),
        route("/campaigns/{id}/npcs", Dm, get(c::campaign_npcs)),
        route("/campaigns/{id}/maps", Dm, get(c::campaign_maps)),
        route("/modules/{id}", Dm, get(c::get_module)),
        route("/modules/{id}/documents", Dm, get(c::module_documents)),
        route("/modules/{id}/monsters", Dm, get(c::module_monsters)),
        route("/modules/{id}/npcs", Dm, get(c::module_npcs)),
        route("/modules/{id}/maps", Dm, get(c::module_maps)),
        route("/documents/{id}", Dm, get(c::get_document)),
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
        let unique: HashSet<_> = table.iter().map(|r| (r.method, r.path)).collect();
        assert_eq!(unique.len(), table.len());
        assert!(table.iter().all(|r| r.path.starts_with('/')));
    }
}
