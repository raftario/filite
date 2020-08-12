use crate::util::DefaultExt;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[inline]
fn default_log_level() -> String {
    "info,sqlx=warn".to_owned()
}
#[inline]
fn log_level_is_default(level: &str) -> bool {
    level.to_lowercase() == default_log_level()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub files_dir: PathBuf,
    #[serde(skip_serializing_if = "log_level_is_default")]
    pub log_level: String,
    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub pool: PoolConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub threads: ThreadsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 80,
            database_url: "filite.db".to_owned(),
            files_dir: PathBuf::from("files"),
            log_level: default_log_level(),
            tls: None,
            pool: Default::default(),
            threads: Default::default(),
        }
    }
}

pub fn read(path: impl AsRef<Path>) -> Result<&'static Config> {
    let file = File::open(path)?;
    let config: Config = serde_json::from_reader(BufReader::new(file))?;
    Ok(&*Box::leak(Box::new(config)))
}

pub fn write(path: impl AsRef<Path>) -> Result<()> {
    let config: Config = Default::default();
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), &config)?;
    Ok(())
}

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

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct ThreadsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_threads: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_threads: Option<usize>,
}
