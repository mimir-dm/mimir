//! `mimir-server`: run the Mimir web server (COLLIERY-I-0612).

use mimir_server::config::Config;
use mimir_server::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mimir-server: {e}");
            std::process::exit(2);
        }
    };
    if config.open_mode() {
        tracing::warn!("MIMIR_API_TOKEN is not set: open mode, the API needs no token");
    }
    if let Err(e) = mimir_server::prepare(&config) {
        eprintln!("mimir-server: {e}");
        std::process::exit(1);
    }

    let bind = config.bind;
    tracing::info!(%bind, data_dir = %config.data_dir.display(), "mimir-server listening");
    let app = mimir_server::router(AppState::new(config))
        .layer(tower_http::trace::TraceLayer::new_for_http());
    let listener = match tokio::net::TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("mimir-server: cannot listen on {bind}: {e}");
            std::process::exit(1);
        }
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server error");
}
