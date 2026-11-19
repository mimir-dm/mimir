//! Dev tools (MIMIR-T-0734): on a development server with the fixture
//! (`MIMIR_SEED=fixture`), the DM seeds, reseeds and clears the fixture
//! campaign from Settings, as the desktop's dev tools did. On any other
//! server these routes answer 404.

use axum::extract::{Path, State};
use axum::Json;
use mimir_core::seed;
use mimir_core::services::ServiceError;
use mimir_wire as wire;

use crate::error::ApiError;
use crate::state::AppState;

fn dev_only(state: &AppState) -> Result<(), ApiError> {
    if state.config.seed_fixture {
        Ok(())
    } else {
        Err(ServiceError::not_found("Dev tools", "fixture").into())
    }
}

/// `GET /dev/fixture`
pub async fn fixture(State(state): State<AppState>) -> Result<Json<wire::DevFixture>, ApiError> {
    dev_only(&state)?;
    state
        .with_db(|conn| {
            Ok(wire::DevFixture {
                seeded: seed::is_already_seeded(conn)?,
            })
        })
        .await
        .map(Json)
}

/// `POST /dev/fixture/{action}`: `seed` (when it is not in), `reseed`
/// (clear, then seed), `clear`.
pub async fn act(
    State(state): State<AppState>,
    Path(action): Path<String>,
) -> Result<Json<wire::DevFixture>, ApiError> {
    dev_only(&state)?;
    let data_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            match action.as_str() {
                "seed" => {
                    seed::seed_dev_data(conn, &data_dir)?;
                }
                "reseed" => {
                    seed::clear_dev_seed_data(conn)?;
                    seed::seed_dev_data(conn, &data_dir)?;
                }
                "clear" => seed::clear_dev_seed_data(conn)?,
                other => {
                    return Err(ServiceError::validation(format!(
                        "unknown action {other:?} (seed, reseed, clear)"
                    ))
                    .into())
                }
            }
            Ok(wire::DevFixture {
                seeded: seed::is_already_seeded(conn)?,
            })
        })
        .await
        .map(Json)
}
