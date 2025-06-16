use std::borrow::Cow;
use std::fs;

use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use sqlx::{Pool, Postgres};
use tracing::info;
use tracing::log::LevelFilter;

use crate::common::manifest::Manifest;

pub const USER_SESSION_KEY: &str = "USER";

#[derive(Deserialize)]
pub struct Db {
    url: Option<String>,
}

impl Db {
    pub fn url(&self) -> Result<&str, crate::error::Configuration> {
        self.url
            .as_deref()
            .ok_or_else(|| crate::error::Configuration::MissingField {
                path: Cow::from("db.url"),
            })
    }
}

#[derive(Deserialize)]
pub struct Configuration {
    pub url: String,
    pub log: String,
    pub password_salt: String,
    pub db: Db,
}

pub async fn default_load_config() -> Result<Configuration, Box<dyn std::error::Error>> {
    let cfg_path = std::option_env!("CONFIG").unwrap_or_else(|| "app.toml");
    info!("Loaded configuration: {:?}", cfg_path);
    let mut config: Configuration = toml::from_str(fs::read_to_string(cfg_path)?.as_str())?;
    if config.url.is_empty() {
        config.url = std::option_env!("DATABASE_URL")
            .expect("DATABASE_URL or db.url must be set")
            .to_string();
    }
    Ok(config)
}
pub async fn init_app_state(
    config: &Configuration,
) -> Result<AppState, Box<dyn std::error::Error>> {
    let db = config.db.url()?;
    let opts: PgConnectOptions = db.parse()?;
    Ok(AppState {
        manifest: serde_json::from_str(include_str!("../static/manifest.json"))
            .expect("Failed to parse manifest"),
        pool: PgPoolOptions::new()
            .connect_with(opts.log_statements(LevelFilter::Trace))
            .await?,
        password_salt: config.password_salt.clone(),
        uri: config.url.clone(),
        crossbeam: CrossBeam::default(),
    })
}

pub type DB = Postgres;
pub struct AppState {
    pub manifest: Manifest,
    pub pool: Pool<DB>,
    pub password_salt: String,
    pub uri: String,
    pub crossbeam: CrossBeam,
}

pub struct CrossBeam {
}

impl CrossBeam {
    pub fn new() -> CrossBeam {
        CrossBeam {
        }
    }
}

impl Default for CrossBeam {
    fn default() -> Self {
        Self::new()
    }
}

