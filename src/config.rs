use crate::util::DefaultExt;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub port: u16,
    pub database: DatabaseConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub runtime: RuntimeConfig,
    #[serde(skip_serializing_if = "DefaultExt::is_default")]
    pub password: PasswordConfig,
    #[serde(skip_serializing_if = "Config::log_level_is_default")]
    pub log_level: String,
}

impl Config {
    #[inline]
    fn default_log_level() -> String {
        "info".to_owned()
    }
    #[inline]
    fn log_level_is_default(level: &str) -> bool {
        level.to_lowercase() == Self::default_log_level()
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            port: 80,
            database: Default::default(),
            tls: None,
            runtime: Default::default(),
            password: Default::default(),
            log_level: Self::default_log_level(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "DefaultExt::is_default")]
    pub mode: DatabaseMode,
    #[serde(
        default = "DatabaseConfig::default_cache_capacity",
        skip_serializing_if = "DatabaseConfig::cache_capacity_is_default"
    )]
    pub cache_capacity: u64,
}

impl DatabaseConfig {
    #[inline]
    fn default_cache_capacity() -> u64 {
        1024 * 1024 * 1024
    }
    #[inline]
    fn cache_capacity_is_default(cc: &u64) -> bool {
        *cc == Self::default_cache_capacity()
    }
}

impl Default for DatabaseConfig {
    #[inline]
    fn default() -> Self {
        Self {
            path: PathBuf::from("filite"),
            mode: Default::default(),
            cache_capacity: Self::default_cache_capacity(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatabaseMode {
    Space,
    Throughput,
}

impl From<DatabaseMode> for sled::Mode {
    #[inline]
    fn from(m: DatabaseMode) -> Self {
        match m {
            DatabaseMode::Space => Self::LowSpace,
            DatabaseMode::Throughput => Self::HighThroughput,
        }
    }
}

impl Default for DatabaseMode {
    #[inline]
    fn default() -> Self {
        Self::Space
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RuntimeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_threads: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_threads: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PasswordConfig {
    pub hash_length: Option<u32>,
    pub salt_length: Option<usize>,
    pub lanes: Option<u32>,
    pub memory_cost: Option<u32>,
    pub time_cost: Option<u32>,
    pub secret: Option<String>,
}
