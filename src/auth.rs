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

pub fn auth(
    db: &'static Db,
    config: &'static Config,
) -> impl Filter<Extract = (User,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::value("Authorization")
        .or(warp::any().and_then(|| async move {
            Result::<HeaderValue, Rejection>::Err(crate::reject::unauthorized(
                "Authentication Required",
            ))
        }))
        .unify()
        .and_then(move |header| user(header, db, config))
}

#[tracing::instrument(level = "debug", skip(db))]
async fn user(header: HeaderValue, db: &Db, config: &Config) -> Result<User, Rejection> {
    let credentials = Basic::decode(&header).or_bad_request("Invalid Credentials")?;

    let user = crate::db::user(credentials.username(), db)
        .or_500()?
        .or_unauthorized("Invalid Credentials")?;

    let valid = !task::block_in_place(|| {
        verify(
            &user.password_hash,
            credentials.password().as_bytes(),
            &config.password,
        )
    })
    .or_500()?;
    if !valid {
        return Err(crate::reject::unauthorized("Invalid Credentials"));
    }

    Ok(user)
}

#[tracing::instrument(level = "debug", skip(encoded, password))]
fn verify(encoded: &str, password: &[u8], config: &PasswordConfig) -> Result<bool> {
    let res = match &config.secret {
        Some(s) => argon2::verify_encoded_ext(encoded, password, s.as_bytes(), &[])?,
        None => argon2::verify_encoded(encoded, password)?,
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
