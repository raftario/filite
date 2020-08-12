pub mod models;
pub mod pool;

use crate::db::models::{Filite, Role, Type, User, Visibility};
use anyhow::Result;
use sqlx::SqlitePool;

#[tracing::instrument(level = "debug")]
pub async fn user(id: &str, pool: &SqlitePool) -> Result<Option<User>> {
    Ok(sqlx::query_as!(
        User,
        r#"SELECT id, password, role as "role: Role" FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?)
}

#[tracing::instrument(level = "debug")]
pub async fn filite(id: &str, view: bool, pool: &SqlitePool) -> Result<Option<Filite>> {
    if !view
        || sqlx::query!("UPDATE filite SET views = views + 1 WHERE id = $1", id)
            .fetch_optional(pool)
            .await?
            .is_some()
    {
        Ok(sqlx::query_as!(
            Filite,
            r#"SELECT id, ty as "ty: Type", val, creator, created, visibility as "visibility: Visibility", views FROM filite WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await?)
    } else {
        Ok(None)
    }
}
