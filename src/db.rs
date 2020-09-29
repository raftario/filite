use crate::config::{Config, DatabaseConfig};
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
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
pub enum Filite {
    File { data: Vec<u8>, mime: String },
    Link { location: String },
    Text { data: String },
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn filite(id: &str, db: &Db) -> Result<Option<Filite>> {
    task::block_in_place(move || {
        let bytes = match db.get(id)? {
            Some(b) => b,
            None => return Ok(None),
        };
        let filite = bincode::deserialize(&bytes)?;
        Ok(Some(filite))
    })
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn create_filite(id: &str, filite: Filite, db: &Db) -> Result<bool> {
    task::block_in_place(move || {
        if db.contains_key(id)? {
            return Ok(false);
        }
        let bytes = bincode::serialize(&filite)?;
        db.insert(id, bytes)?;
        Ok(true)
    })
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn random_id(length: usize, db: &Db) -> Result<String> {
    task::block_in_place(move || {
        let mut id;
        loop {
            id = rand::thread_rng()
                .sample_iter(Alphanumeric)
                .take(length)
                .collect();
            if !db.contains_key(&id)? {
                break Ok(id);
            }
        }
    })
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

#[tracing::instrument(level = "debug", skip(password, db))]
pub fn create_user(
    id: &str,
    password: &str,
    admin: bool,
    db: &Db,
    config: &Config,
) -> Result<bool> {
    task::block_in_place(move || {
        let users = db.open_tree("users")?;
        if users.contains_key(id)? {
            return Ok(false);
        }

        let password_hash = crate::auth::hash(password.as_bytes(), &config.password)?;
        let user = User {
            admin,
            password_hash,
        };

        let bytes = bincode::serialize(&user)?;
        users.insert(id, bytes)?;
        Ok(true)
    })
}
