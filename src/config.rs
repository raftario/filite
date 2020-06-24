use anyhow::Error;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub port: u16,
    pub database_url: String,

    #[serde(default)]
    pub logger: LoggerConfig,

    #[serde(default)]
    pub pool: PoolConfig,

    #[cfg(feature = "tls")]
    #[serde(default)]
    pub tls: TlsConfig,

    #[cfg(feature = "threaded")]
    #[serde(default)]
    pub threads: ThreadsConfig,

    #[cfg(feature = "analytics")]
    #[serde(default)]
    pub analytics: AnalyticsConfig,

    #[cfg(feature = "highlight")]
    #[serde(default)]
    pub highlight: HighlightConfig,
}

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let config = serde_json::from_reader(BufReader::new(file))?;
        Ok(config)
    }

    pub fn write(path: impl AsRef<Path>) -> Result<(), Error> {
        let config = Self {
            port: 80,
            database_url: {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "sqlite")] {
                        "filite.db"
                    } else if #[cfg(feature = "postgres")] {
                        "postgresql://localhost:5432/filite"
                    } else if #[cfg(feature = "mysql")] {
                        "mysql://localhost:3306/filite"
                    }
                }
            }
            .to_owned(),
            logger: Default::default(),
            pool: Default::default(),
            #[cfg(feature = "tls")]
            tls: Default::default(),
            #[cfg(feature = "threaded")]
            threads: Default::default(),
            #[cfg(feature = "analytics")]
            analytics: Default::default(),
            #[cfg(feature = "highlight")]
            highlight: Default::default(),
        };
        let file = File::create(path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), &config)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct LoggerConfig {
    pub console: LogLevel,
    pub file: Option<FileLoggerConfig>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLoggerConfig {
    #[serde(default)]
    pub level: LogLevel,
    pub path: PathBuf,
}
#[derive(Deserialize, Serialize)]
pub struct LogLevel(pub LevelFilter);
impl Default for LogLevel {
    fn default() -> Self {
        Self(LevelFilter::Info)
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PoolConfig {
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
    pub connect_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
    pub max_lifetime: Option<u64>,
}

#[cfg(feature = "tls")]
#[derive(Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
}

#[cfg(feature = "threaded")]
#[derive(Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ThreadsConfig {
    pub core_threads: Option<usize>,
    pub max_threads: Option<usize>,
}

#[cfg(feature = "analytics")]
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AnalyticsConfig {
    pub views: bool,
}

#[cfg(feature = "highlight")]
#[derive(Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct HighlightConfig {
    pub theme: String,
    pub languages: Vec<String>,
}
#[cfg(feature = "highlight")]
impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_owned(),
            languages: Default::default(),
        }
    }
}
