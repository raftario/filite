use crate::config::{Config, PoolConfig};
use anyhow::Error;
use sqlx::{pool::Builder, Connect};
use std::time::Duration;

pub enum Pool {
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::PgPool),
    #[cfg(feature = "mysql")]
    MySql(sqlx::MySqlPool),
}

impl Pool {
    pub async fn build(config: &Config) -> Result<Self, Error> {
        if config.database_url.starts_with("postgresql://") {
            cfg_if::cfg_if! {
                if #[cfg(feature = "postgres")] {
                    let pool = Self::apply_config(sqlx::PgPool::builder(), &config.pool);
                    Ok(Self::Postgres(pool.build(&config.database_url).await?))
                } else {
                    Err(anyhow::anyhow!("This build of filite doesn't support PostgreSQL"))
                }
            }
        } else if config.database_url.starts_with("mysql://") {
            cfg_if::cfg_if! {
                if #[cfg(feature = "mysql")] {
                    let pool = Self::apply_config(sqlx::MySqlPool::builder(), &config.pool);
                    Ok(Self::MySql(pool.build(&config.database_url).await?))
                } else {
                    Err(anyhow::anyhow!("This build of filite doesn't support MySQL"))
                }
            }
        } else {
            cfg_if::cfg_if! {
                if #[cfg(feature = "sqlite")] {
                    let pool = Self::apply_config(sqlx::SqlitePool::builder(), &config.pool);
                    Ok(Self::Sqlite(pool.build(&config.database_url).await?))
                } else {
                    Err(anyhow::anyhow!("This build of filite doesn't support SQLite"))
                }
            }
        }
    }

    fn apply_config<C: Connect>(mut builder: Builder<C>, config: &PoolConfig) -> Builder<C> {
        if let Some(ms) = config.max_size {
            builder = builder.max_size(ms);
        }
        if let Some(ms) = config.min_size {
            builder = builder.min_size(ms);
        }
        if let Some(ct) = config.connect_timeout {
            builder = builder.connect_timeout(Duration::from_millis(ct));
        }
        if let Some(it) = config.idle_timeout {
            builder = builder.idle_timeout(Duration::from_millis(it));
        }
        if let Some(ml) = config.max_lifetime {
            builder = builder.max_lifetime(Duration::from_millis(ml));
        }
        builder
    }
}
