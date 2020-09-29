use crate::config::DatabaseConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use tokio::task;

#[tracing::instrument(level = "debug")]
pub fn connect(config: &DatabaseConfig) -> Result<&'static Db> {
    let db = sled::Config::default()
        .path(&config.path)
        .mode(config.mode.into())
        .cache_capacity(config.cache_capacity)
        .open()?;
    Ok(Box::leak(Box::new(db)))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub admin: bool,
    pub password_hash: String,
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn user(id: &str, db: &Db) -> Result<Option<User>> {
    task::block_in_place(move || {
        let users = db.open_tree("users")?;
        let bytes = match users.get(id)? {
            Some(b) => b,
            None => return Ok(None),
        };
        let user = bincode::deserialize(&bytes)?;
        Ok(Some(user))
    })
}
