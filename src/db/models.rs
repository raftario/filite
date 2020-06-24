use anyhow::{anyhow, Error};
// use chrono::{DateTime, Utc};
use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};
use tokio::task;

#[derive(sqlx::FromRow)]
pub struct FiliteRow {
    id: String,
    ty: i32,
    val: String,
    creator: String,
    // created: DateTime<Utc>,
    visibility: i32,
    views: i32,
}

pub enum Filite {
    File {
        id: String,
        path: PathBuf,
        creator: String,
        // created: DateTime<Utc>,
        visibility: Visibility,
        #[cfg(feature = "analytics")]
        views: i32,
    },
    Link {
        id: String,
        url: String,
        creator: String,
        // created: DateTime<Utc>,
        visibility: Visibility,
        #[cfg(feature = "analytics")]
        views: i32,
    },
    Text {
        id: String,
        contents: String,
        creator: String,
        // created: DateTime<Utc>,
        visibility: Visibility,
        #[cfg(feature = "analytics")]
        views: i32,
    },
}

pub enum Visibility {
    Public,
    Internal,
    Private,
}

impl TryFrom<FiliteRow> for Filite {
    type Error = Error;
    fn try_from(row: FiliteRow) -> Result<Self, Self::Error> {
        match row.ty {
            0 => Ok(Filite::File {
                id: row.id,
                path: PathBuf::from(row.val),
                creator: row.creator,
                // created: row.created,
                visibility: row.visibility.try_into()?,
                #[cfg(feature = "analytics")]
                views: row.views,
            }),
            1 => Ok(Filite::Link {
                id: row.id,
                url: row.val,
                creator: row.creator,
                // created: row.created,
                visibility: row.visibility.try_into()?,
                #[cfg(feature = "analytics")]
                views: row.views,
            }),
            2 => Ok(Filite::Text {
                id: row.id,
                contents: row.val,
                creator: row.creator,
                // created: row.created,
                visibility: row.visibility.try_into()?,
                #[cfg(feature = "analytics")]
                views: row.views,
            }),
            ty => Err(anyhow!("unknown type {}", ty)),
        }
    }
}

impl TryFrom<i32> for Visibility {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Visibility::Public),
            1 => Ok(Visibility::Internal),
            2 => Ok(Visibility::Private),
            _ => Err(anyhow!("unknown visibility {}", value)),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
    username: String,
    password: String,
    role: i32,
    // registered: DateTime<Utc>,
}

pub struct User {
    pub username: String,
    pub password: String,
    pub role: Role,
    // pub registered: DateTime<Utc>,
}

pub enum Role {
    User,
    Admin,
}

impl TryFrom<UserRow> for User {
    type Error = Error;
    fn try_from(value: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            username: value.username,
            password: value.password,
            role: value.role.try_into()?,
        })
    }
}

impl TryFrom<i32> for Role {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Role::User),
            1 => Ok(Role::Admin),
            _ => Err(anyhow!("unknown role {}", value)),
        }
    }
}

impl User {
    pub async fn verify_password(&self, password: &str) -> Result<bool, Error> {
        let encoded = self.password.clone();
        let pwd = password.as_bytes().to_vec();
        Ok(task::spawn_blocking(move || argon2::verify_encoded(&encoded, &pwd)).await??)
    }
}
