//! Server configuration from the environment (MIMIR-T-0704).

use std::net::SocketAddr;
use std::path::PathBuf;

/// Everything the server reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// `MIMIR_BIND` — address to listen on (default `127.0.0.1:8740`; the
    /// container sets `0.0.0.0:8740`).
    pub bind: SocketAddr,
    /// `MIMIR_DATA_DIR` — the app directory: `data/mimir.db` and `assets/`
    /// live under it (default `./mimir-data`).
    pub data_dir: PathBuf,
    /// `MIMIR_API_TOKEN` — the DM bearer token. Unset or empty: open mode.
    pub api_token: Option<String>,
    /// `MIMIR_WEB_DIST` — serve the built web app from this directory
    /// (development: `crates/mimir-web/dist`).
    pub web_dist: Option<PathBuf>,
    /// `MIMIR_SEED=fixture` — seed the SRD catalog and the fixture campaign
    /// into an empty database. For development and tests only.
    pub seed_fixture: bool,
}

/// Default listen address.
pub const DEFAULT_BIND: &str = "127.0.0.1:8740";

impl Config {
    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Read the configuration through a lookup function (testable).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let bind = get("MIMIR_BIND").unwrap_or_else(|| DEFAULT_BIND.to_string());
        let bind = bind
            .parse()
            .map_err(|_| format!("MIMIR_BIND is not an address: {bind}"))?;
        let seed = get("MIMIR_SEED").unwrap_or_default();
        let seed_fixture = match seed.as_str() {
            "" => false,
            "fixture" => true,
            other => {
                return Err(format!(
                    "MIMIR_SEED must be empty or \"fixture\", not {other:?}"
                ))
            }
        };
        Ok(Self {
            bind,
            data_dir: get("MIMIR_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("mimir-data")),
            api_token: get("MIMIR_API_TOKEN").filter(|t| !t.trim().is_empty()),
            web_dist: get("MIMIR_WEB_DIST")
                .filter(|d| !d.is_empty())
                .map(PathBuf::from),
            seed_fixture,
        })
    }

    /// The SQLite database file.
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("data").join("mimir.db")
    }

    /// Campaign assets (maps, images).
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.join("assets")
    }

    /// Open mode: no token configured, every route is open.
    pub fn open_mode(&self) -> bool {
        self.api_token.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cfg(pairs: &[(&str, &str)]) -> Result<Config, String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config::from_lookup(|k| map.get(k).cloned())
    }

    #[test]
    fn defaults_are_local_and_open() {
        let c = cfg(&[]).unwrap();
        assert_eq!(c.bind.to_string(), DEFAULT_BIND);
        assert!(c.open_mode());
        assert!(!c.seed_fixture);
        assert_eq!(c.database_path(), PathBuf::from("mimir-data/data/mimir.db"));
        assert_eq!(c.web_dist, None);
    }

    #[test]
    fn reads_every_variable() {
        let c = cfg(&[
            ("MIMIR_BIND", "0.0.0.0:9000"),
            ("MIMIR_DATA_DIR", "/srv/mimir"),
            ("MIMIR_API_TOKEN", "secret"),
            ("MIMIR_WEB_DIST", "crates/mimir-web/dist"),
            ("MIMIR_SEED", "fixture"),
        ])
        .unwrap();
        assert_eq!(c.bind.port(), 9000);
        assert_eq!(c.assets_dir(), PathBuf::from("/srv/mimir/assets"));
        assert_eq!(c.api_token.as_deref(), Some("secret"));
        assert!(!c.open_mode());
        assert!(c.seed_fixture);
        assert_eq!(c.web_dist, Some(PathBuf::from("crates/mimir-web/dist")));
    }

    #[test]
    fn a_blank_token_is_open_mode_and_bad_values_are_errors() {
        assert!(cfg(&[("MIMIR_API_TOKEN", "  ")]).unwrap().open_mode());
        assert!(cfg(&[("MIMIR_BIND", "nope")]).is_err());
        assert!(cfg(&[("MIMIR_SEED", "prod")]).is_err());
    }
}
