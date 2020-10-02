use crate::config::{Config, DatabaseConfig};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fmt;

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
pub struct Filite {
    pub owner: String,
    pub creation: DateTime<Utc>,
    pub inner: FiliteInner,
    pub views: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FiliteInner {
    File { data: Vec<u8>, mime: String },
    Link { location: String },
    Text { data: String },
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn get(id: &str, db: &Db) -> Result<Option<Filite>> {
    macro_rules! tryy {
        ($op:expr, $default:expr, $val:ident) => {
            match $op {
                Ok(r) => r,
                Err(e) => {
                    $val = Err(e.into());
                    return Some($default.to_owned());
                }
            }
        };
    }

    let mut filite = Ok(None);
    db.fetch_and_update(id, |b| match b {
        Some(b) => {
            let mut f: Filite = tryy!(bincode::deserialize(b), b, filite);
            f.views += 1;

            let b = tryy!(bincode::serialize(&f), b, filite);
            filite = Ok(Some(f));
            Some(b)
        }
        None => None,
    })?;
    filite
}

fn insert_filite(id: &str, filite: Filite, db: &Db) -> Result<Option<Filite>> {
    if db.contains_key(id)? {
        return Ok(None);
    }

    let bytes = bincode::serialize(&filite)?;
    db.insert(id, bytes)?;
    Ok(Some(filite))
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn insert_file(
    id: &str,
    owner: String,
    data: Vec<u8>,
    mime: String,
    db: &Db,
) -> Result<Option<Filite>> {
    insert_filite(
        id,
        Filite {
            owner,
            creation: Utc::now(),
            inner: FiliteInner::File { data, mime },
            views: 0,
        },
        db,
    )
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn insert_link(id: &str, owner: String, location: String, db: &Db) -> Result<Option<Filite>> {
    insert_filite(
        id,
        Filite {
            owner,
            creation: Utc::now(),
            inner: FiliteInner::Link { location },
            views: 0,
        },
        db,
    )
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn insert_text(id: &str, owner: String, data: String, db: &Db) -> Result<Option<Filite>> {
    insert_filite(
        id,
        Filite {
            owner,
            creation: Utc::now(),
            inner: FiliteInner::Text { data },
            views: 0,
        },
        db,
    )
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn delete_filite(id: &str, user: &User, db: &Db) -> Result<Option<Filite>> {
    let bytes = match db.get(id)? {
        Some(b) => b,
        None => return Ok(None),
    };
    let filite: Filite = bincode::deserialize(&bytes)?;

    if user.admin || filite.owner == user.id {
        db.remove(id)?;
        Ok(Some(filite))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn random_id(length: usize, db: &Db) -> Result<String> {
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
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub admin: bool,
    pub password_hash: String,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("admin", &self.admin)
            .finish()
    }
}

#[derive(Deserialize, Serialize)]
struct DbUser {
    admin: bool,
    password_hash: String,
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn user(id: &str, db: &Db) -> Result<Option<User>> {
    let users = db.open_tree("users")?;
    let bytes = match users.get(id)? {
        Some(b) => b,
        None => return Ok(None),
    };
    let user: DbUser = bincode::deserialize(&bytes)?;
    Ok(Some(User {
        id: id.to_owned(),
        admin: user.admin,
        password_hash: user.password_hash,
    }))
}

#[tracing::instrument(level = "debug", skip(password, db))]
pub fn insert_user(
    id: &str,
    password: &str,
    admin: bool,
    db: &Db,
    config: &Config,
) -> Result<Option<User>> {
    let users = db.open_tree("users")?;
    if users.contains_key(id)? {
        return Ok(None);
    }

    let password_hash = crate::auth::hash(password.as_bytes(), &config.password)?;
    let user = DbUser {
        admin,
        password_hash,
    };

    let bytes = bincode::serialize(&user)?;
    users.insert(id, bytes)?;
    Ok(Some(User {
        id: id.to_owned(),
        admin: user.admin,
        password_hash: user.password_hash,
    }))
}

#[tracing::instrument(level = "debug", skip(db))]
pub fn delete_user(id: &str, db: &Db) -> Result<Option<User>> {
    let users = db.open_tree("users")?;
    match users.remove(id)? {
        Some(b) => {
            let user: DbUser = bincode::deserialize(&b)?;
            Ok(Some(User {
                id: id.to_owned(),
                admin: user.admin,
                password_hash: user.password_hash,
            }))
        }
        None => Ok(None),
    }
}
