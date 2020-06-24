pub mod models;
pub mod pool;

use anyhow::Error;
use futures_core::Stream;
use futures_util::StreamExt;
use models::{Filite, FiliteRow, User, UserRow};
use pool::Pool;
use std::{convert::TryInto, pin::Pin};

pub async fn fetch(id: &str, pool: &Pool) -> Result<Option<Filite>, Error> {
    let sql = "SELECT * FROM data WHERE id = ?";
    let row: Option<FiliteRow> = match pool {
        #[cfg(feature = "sqlite")]
        Pool::Sqlite(p) => {
            use sqlx::sqlite::SqliteQueryAs;
            sqlx::query_as(sql).bind(id).fetch_optional(p).await?
        }
        #[cfg(feature = "postgres")]
        Pool::Postgres(p) => {
            use sqlx::postgres::PgQueryAs;
            sqlx::query_as(sql).bind(id).fetch_optional(p).await?
        }
        #[cfg(feature = "mysql")]
        Pool::MySql(p) => {
            use sqlx::mysql::MySqlQueryAs;
            sqlx::query_as(sql).bind(id).fetch_optional(p).await?
        }
    };
    let filite: Option<Filite> = match row {
        Some(row) => Some(row.try_into()?),
        None => None,
    };
    Ok(filite)
}

pub fn fetch_all<'a>(pool: &'a Pool) -> impl Stream<Item = Result<Filite, Error>> + 'a {
    let sql = "SELECT * FROM data";
    let rows: Pin<Box<dyn Stream<Item = Result<FiliteRow, sqlx::Error>>>> = match pool {
        #[cfg(feature = "sqlite")]
        Pool::Sqlite(p) => {
            use sqlx::sqlite::SqliteQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
        #[cfg(feature = "postgres")]
        Pool::Postgres(p) => {
            use sqlx::postgres::PgQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
        #[cfg(feature = "mysql")]
        Pool::MySql(p) => {
            use sqlx::mysql::MySqlQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
    };
    rows.map(|r| match r {
        Ok(r) => r.try_into(),
        Err(e) => Err(e.into()),
    })
}

pub async fn fetch_user(username: &str, pool: &Pool) -> Result<Option<User>, Error> {
    let sql = "SELECT * FROM users WHERE username = ?";
    let row: Option<UserRow> = match pool {
        #[cfg(feature = "sqlite")]
        Pool::Sqlite(p) => {
            use sqlx::sqlite::SqliteQueryAs;
            sqlx::query_as(sql).bind(username).fetch_optional(p).await?
        }
        #[cfg(feature = "postgres")]
        Pool::Postgres(p) => {
            use sqlx::postgres::PgQueryAs;
            sqlx::query_as(sql).bind(username).fetch_optional(p).await?
        }
        #[cfg(feature = "mysql")]
        Pool::MySql(p) => {
            use sqlx::mysql::MySqlQueryAs;
            sqlx::query_as(sql).bind(username).fetch_optional(p).await?
        }
    };
    let user: Option<User> = match row {
        Some(row) => Some(row.try_into()?),
        None => None,
    };
    Ok(user)
}

pub fn fetch_all_users<'a>(pool: &'a Pool) -> impl Stream<Item = Result<User, Error>> + 'a {
    let sql = "SELECT * FROM users";
    let rows: Pin<Box<dyn Stream<Item = Result<UserRow, sqlx::Error>>>> = match pool {
        #[cfg(feature = "sqlite")]
        Pool::Sqlite(p) => {
            use sqlx::sqlite::SqliteQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
        #[cfg(feature = "postgres")]
        Pool::Postgres(p) => {
            use sqlx::postgres::PgQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
        #[cfg(feature = "mysql")]
        Pool::MySql(p) => {
            use sqlx::mysql::MySqlQueryAs;
            sqlx::query_as(sql).fetch(p)
        }
    };
    rows.map(|r| match r {
        Ok(r) => r.try_into(),
        Err(e) => Err(e.into()),
    })
}
