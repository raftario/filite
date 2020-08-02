use crate::utils::DefaultExt;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

fn log_level_is_info(level: &str) -> bool {
    level.to_lowercase() == "info"
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub port: u16,
    pub database_url: String,

    #[serde(skip_serializing_if = "log_level_is_info")]
    pub log_level: String,

    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub pool: PoolConfig,

    #[cfg(feature = "tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,

    #[cfg(feature = "threaded")]
    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub threads: ThreadsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 80,
            database_url: {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "sqlite")] {
                        "sqlite://filite.db"
                    } else if #[cfg(feature = "postgres")] {
                        "postgresql://localhost:5432/filite"
                    }
                }
            }
            .to_owned(),

            log_level: "info".to_owned(),

            #[cfg(feature = "tls")]
            tls: None,

            pool: Default::default(),

            #[cfg(feature = "threaded")]
            threads: Default::default(),
        }
    }
}

pub fn read(path: impl AsRef<Path>) -> Result<&'static Config, Error> {
    let file = File::open(path)?;
    let config: Config = serde_json::from_reader(BufReader::new(file))?;
    Ok(&*Box::leak(Box::new(config)))
}

pub fn write(path: impl AsRef<Path>) -> Result<(), Error> {
    let config: Config = Default::default();
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), &config)?;
    Ok(())
}

#[cfg(feature = "tls")]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct PoolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_connections: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lifetime: Option<u64>,
}

#[cfg(feature = "threaded")]
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct ThreadsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_threads: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_threads: Option<usize>,
}
