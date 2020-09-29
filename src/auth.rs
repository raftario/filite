use crate::{
    config::{Config, PasswordConfig},
    db::{self, User},
    reject::{self, TryExt},
};
use anyhow::Result;
use rand::Rng;
use sled::Db;
use tokio::task;
use warp::{Filter, Rejection};

pub fn auth_optional(
    db: &'static Db,
    config: &'static Config,
) -> impl Filter<Extract = (Option<User>,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::optional("Authorization").and_then(move |header| async move {
        match header {
            Some(h) => match user(h, db, config).await {
                Ok(u) => Ok(Some(u)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    })
}

pub fn auth_required(
    db: &'static Db,
    config: &'static Config,
) -> impl Filter<Extract = (User,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::header("Authorization").and_then(move |header| user(header, db, config))
}

#[tracing::instrument(level = "debug")]
async fn user(header: String, db: &Db, config: &Config) -> Result<User, Rejection> {
    if &header[..5] != "Basic" {
        return Err(reject::unauthorized());
    }

    let decoded = task::block_in_place(move || base64::decode(&header[6..])).or_401()?;

    let (user, password) = {
        let mut split = None;
        for (i, b) in decoded.iter().copied().enumerate() {
            if b == b':' {
                split = Some(i);
            }
        }
        let split = split.or_401()?;

        let (u, p) = (&decoded[..split], &decoded[(split + 1)..]);
        (std::str::from_utf8(u).or_401()?, p)
    };

    let user = db::user(user, db).or_500()?.or_401()?;
    if !verify(&user.password_hash, password, &config.password).or_500()? {
        return Err(reject::unauthorized());
    }

    Ok(user)
}

#[tracing::instrument(level = "debug", skip(password))]
fn hash(password: &[u8], config: &PasswordConfig) -> Result<String> {
    let mut cfg = argon2::Config::default();
    if let Some(hl) = config.hash_length {
        cfg.hash_length = hl;
    }
    if let Some(l) = config.lanes {
        cfg.lanes = l;
    }
    if let Some(mc) = config.memory_cost {
        cfg.mem_cost = mc;
    }
    if let Some(tc) = config.time_cost {
        cfg.time_cost = tc;
    }
    if let Some(s) = config.secret.as_ref().map(|s| s.as_bytes()) {
        cfg.secret = s;
    }

    let hashed = task::block_in_place(move || {
        let mut salt = vec![0; config.salt_length.unwrap_or(16)];
        rand::thread_rng().fill(&mut salt[..]);

        argon2::hash_encoded(password, &salt[..], &cfg)
    })?;
    Ok(hashed)
}

#[tracing::instrument(level = "debug", skip(encoded, password))]
fn verify(encoded: &str, password: &[u8], config: &PasswordConfig) -> Result<bool> {
    let res = match &config.secret {
        Some(s) => task::block_in_place(move || {
            argon2::verify_encoded_ext(encoded, password, s.as_bytes(), &[])
        })?,
        None => task::block_in_place(move || argon2::verify_encoded(encoded, password))?,
    };
    Ok(res)
}
