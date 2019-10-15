//! Utilities used during the initial setup

use crate::Pool;

use actix_web::middleware::Logger;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use std::env;
use std::path::PathBuf;

#[cfg(debug_assertions)]
use dotenv;
#[cfg(not(debug_assertions))]
use std::fs;
#[cfg(debug_assertions)]
use std::str::FromStr;

/// Returns a path to the directory storing application data
pub fn get_data_dir() -> PathBuf {
    let mut dir = dirs::home_dir().expect("Can't find home directory.");
    dir.push(".filite");
    dir
}

/// Returns a path to the configuration file
#[cfg(not(debug_assertions))]
fn get_config_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("config.toml");
    path
}

/// Returns an environment variable and panic if it isn't found
macro_rules! get_env {
    ($k:literal) => {
        env::var($k).expect(&format!("Can't find {} environment variable.", $k));
    };
}

/// Returns a parsed environment variable and panic if it isn't found or is not parsable
macro_rules! parse_env {
    ($k:literal) => {
        get_env!($k).parse().expect(&format!("Invalid {}.", $k))
    };
}

/// Application configuration
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(not(debug_assertions), serde(default))]
pub struct Config {
    /// Port to listen on
    pub port: u16,
    /// SQLite database connection url
    pub database_url: String,
    /// SQLite database connection pool size
    pub pool_size: u32,
    /// Directory where to store static files
    pub files_dir: PathBuf,
    /// Maximum allowed file size
    pub max_filesize: usize,
}

#[cfg(not(debug_assertions))]
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
            path.push("files");
            path
        };
        let max_filesize = 10_000_000;

        Config {
            port,
            database_url,
            pool_size,
            files_dir,
            max_filesize,
        }
    }
}

impl Config {
    /// Deserialize the config file
    #[cfg(not(debug_assertions))]
    pub fn read_file() -> Result<Self, &'static str> {
        let path = get_config_path();
        let contents = if let Ok(contents) = fs::read_to_string(&path) {
            contents
        } else {
            return Err("Can't read config file.");
        };
        let result = toml::from_str(&contents);

        if result.is_err() {
            return Err("Invalid config file.");
        }
        let mut result: Config = result.unwrap();

        if result.files_dir.is_absolute() {
            if let Err(_) = fs::create_dir_all(&result.files_dir) {
                return Err("Can't create files_dir.");
            }

            result.files_dir = match result.files_dir.canonicalize() {
                Ok(path) => path,
                Err(_) => return Err("Invalid files_dir."),
            }
        } else {
            let mut data_dir = get_data_dir();
            data_dir.push(&result.files_dir);

            if let Err(_) = fs::create_dir_all(&data_dir) {
                return Err("Can't create files_dir.");
            }

            result.files_dir = match data_dir.canonicalize() {
                Ok(path) => path,
                Err(_) => return Err("Invalid files_dir."),
            }
        }

        Ok(result)
    }

    /// Serialize the config file
    #[cfg(not(debug_assertions))]
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

        let port = parse_env!("PORT");
        let database_url = get_env!("DATABASE_URL");
        let pool_size = parse_env!("POOL_SIZE");
        let files_dir = {
            let files_dir = get_env!("FILES_DIR");
            let path = PathBuf::from_str(&files_dir).expect("Can't convert files dir to path");
            if path.is_absolute() {
                path.canonicalize().expect("Invalid FILES_DIR")
            } else {
                let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
                let mut cargo_manifest_dir = PathBuf::from_str(cargo_manifest_dir)
                    .expect("Can't convert cargo manifest dir to path.");
                cargo_manifest_dir.push(&path);
                cargo_manifest_dir
                    .canonicalize()
                    .expect("Invalid FILES_DIR")
            }
        };
        let max_filesize = parse_env!("MAX_FILESIZE");

        Config {
            port,
            database_url,
            pool_size,
            files_dir,
            max_filesize,
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

/// Initializes the logger
pub fn init_logger() {
    if cfg!(debug_assertions) && env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "actix_web=debug");
    } else if !cfg!(debug_assertions) {
        env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
}

/// Returns the logger middleware
pub fn logger_middleware() -> Logger {
    #[cfg(debug_assertions)]
    {
        dotenv::dotenv().ok();
        if let Ok(format) = env::var("LOG_FORMAT") {
            Logger::new(&format)
        } else {
            Logger::default()
        }
    }

    #[cfg(not(debug_assertions))]
    {
        Logger::default()
    }
}
