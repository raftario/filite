use crate::{
    db,
    db::models::User,
    reject::{self, TryExt},
};
use anyhow::Result;
use argon2::Config;
use rand::Rng;
use sqlx::SqlitePool;
use tokio::task;
use warp::{Filter, Rejection};

pub fn auth_optional(
    pool: &'static SqlitePool,
) -> impl Filter<Extract = (Option<User>,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::optional("Authorization").and_then(move |header| async move {
        match header {
            Some(h) => match user(h, pool).await {
                Ok(u) => Ok(Some(u)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    })
}

pub fn auth_required(
    pool: &'static SqlitePool,
) -> impl Filter<Extract = (User,), Error = Rejection> + Copy + Send + Sync + 'static {
    warp::header::header("Authorization").and_then(move |header| user(header, pool))
}

#[tracing::instrument(level = "debug")]
async fn user(header: String, pool: &SqlitePool) -> Result<User, Rejection> {
    if &header[..5] != "Basic" {
        return Err(reject::unauthorized());
    }

    let decoded = base64::decode(&header[6..]).or_401()?;

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

    let user = db::user(user, pool).await.or_500()?.or_401()?;
    if !verify(user.password.clone(), password.to_owned())
        .await
        .or_500()?
    {
        return Err(reject::unauthorized());
    }

    Ok(user)
}

// TODO: Allow custom configuration
#[tracing::instrument(level = "debug", skip(password))]
async fn hash(password: Vec<u8>) -> Result<String> {
    let config = Config::default();
    Ok(task::spawn_blocking(move || {
        let salt: [u8; 16] = rand::thread_rng().gen();
        argon2::hash_encoded(&password, &salt[..], &config)
    })
    .await??)
}

#[tracing::instrument(level = "debug", skip(encoded, password))]
async fn verify(encoded: String, password: Vec<u8>) -> Result<bool> {
    Ok(task::spawn_blocking(move || argon2::verify_encoded(&encoded, &password)).await??)
}
