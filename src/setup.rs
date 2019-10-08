//! Utilities used during the initial setup

use crate::Pool;

use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use std::fs;
use std::path::PathBuf;

#[cfg(debug_assertions)]
use dotenv;
#[cfg(debug_assertions)]
use std::env;
#[cfg(debug_assertions)]
use std::str::FromStr;

/// Returns a path to the directory storing application data
pub fn get_data_dir() -> PathBuf {
    let mut dir = dirs::home_dir().expect("Can't find home directory.");
    dir.push(".filite");
    dir
}

/// Returns a path to the configuration file
fn get_config_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("config.toml");
    path
}

/// Application configuration
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Port to listen on
    port: u16,
    /// SQLite database connection url
    database_url: String,
    /// SQLite database connection pool size
    pool_size: u32,
    /// Directory where to store static files
    files_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let port = 8080;
        let database_url = {
            let mut path = get_data_dir();
            path.push("database.db");
            path.to_str()
                .expect("Can't convert database path to string.")
                .to_owned()
        };
        let pool_size = num_cpus::get() as u32 / 2;
        let files_dir = {
            let mut path = get_data_dir();
            path.push("data");
            path
        };

        Config {
            port,
            database_url,
            pool_size,
            files_dir,
        }
    }
}

impl Config {
    /// Deserialize the config file
    pub fn read_file() -> Result<Self, &'static str> {
        let path = get_config_path();
        let contents = if let Ok(contents) = fs::read_to_string(&path) {
            contents
        } else {
            return Err("Can't read config file.");
        };
        let result = toml::from_str(&contents);
        match result {
            Ok(result) => Ok(result),
            Err(_) => Err("Invalid config file."),
        }
    }

    /// Serialize the config file
    pub fn write_file(&self) -> Result<(), &'static str> {
        let path = get_config_path();
        let contents = toml::to_string(&self).expect("Can't serialize config.");
        match fs::write(&path, &contents) {
            Ok(_) => Ok(()),
            Err(_) => Err("Can't write config file."),
        }
    }

    /// Creates a config from environment variables
    #[cfg(debug_assertions)]
    pub fn debug() -> Self {
        dotenv::dotenv().ok();

        let get_env = |k: &str| -> String {
            env::var(k).expect(&format!("Can't parse {} environment variable.", k))
        };

        let port = get_env("PORT").parse().expect("Invalid PORT.");
        let database_url = get_env("DATABASE_URL");
        let pool_size = get_env("POOL_SIZE").parse().expect("Invalid POOL_SIZE.");
        let files_dir = {
            let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
            let mut path = PathBuf::from_str(cargo_manifest_dir)
                .expect("Can't convert cargo manifest dir to path.");
            let files_dir = get_env("FILES_DIR");
            path.push(&files_dir);
            path
        };

        Config {
            port,
            database_url,
            pool_size,
            files_dir,
        }
    }
}

/// Creates a SQLite database connection pool
pub fn create_pool(url: &str, size: u32) -> Pool {
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    r2d2::Pool::builder()
        .max_size(size)
        .build(manager)
        .expect("Can't create pool.")
}
