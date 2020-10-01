use crate::{
    config::{Config, PasswordConfig},
    db::User,
    reject::TryExt,
};
use anyhow::Result;
use headers::authorization::{Basic, Credentials};
use rand::Rng;
use sled::Db;
use tokio::task;
use warp::{http::HeaderValue, Filter, Rejection};

pub fn optional(
    db: &'static Db,
    config: &'static Config,
) -> impl Filter<Extract = (Option<User>,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::value("Authorization")
        .and_then(move |header| async move {
            match user(header, db, config) {
                Ok(u) => Ok(Some(u)),
                Err(e) => Err(e),
            }
        })
        .or(warp::any().and_then(|| async move { Result::<_, Rejection>::Ok(None) }))
        .unify()
}

pub fn required(
    db: &'static Db,
    config: &'static Config,
) -> impl Filter<Extract = (User,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::value("Authorization")
        .and_then(move |header| async move { user(header, db, config) })
}

#[tracing::instrument(level = "debug", skip(db))]
fn user(header: HeaderValue, db: &Db, config: &Config) -> Result<User, Rejection> {
    let credentials = Basic::decode(&header).or_400()?;

    let user = crate::db::user(credentials.username(), db)
        .or_500()?
        .or_401()?;
    if !verify(
        &user.password_hash,
        credentials.password().as_bytes(),
        &config.password,
    )
    .or_500()?
    {
        return Err(crate::reject::unauthorized());
    }

    Ok(user)
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

#[tracing::instrument(level = "debug", skip(password))]
pub fn hash(password: &[u8], config: &PasswordConfig) -> Result<String> {
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

    let mut salt = vec![0; config.salt_length.unwrap_or(16)];
    rand::thread_rng().fill(&mut salt[..]);

    let hashed = argon2::hash_encoded(password, &salt[..], &cfg)?;
    Ok(hashed)
}
