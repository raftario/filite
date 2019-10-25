//! Utilities used during the initial setup

use crate::Pool;
use actix_web::middleware::Logger;
use blake2::{Blake2b, Digest};
use diesel::{
    r2d2::{self, ConnectionManager},
    sqlite::SqliteConnection,
};
use std::{env, path::PathBuf};

#[cfg(not(feature = "dev"))]
use dirs;
#[cfg(feature = "dev")]
use dotenv;
#[cfg(feature = "dev")]
use std::str::FromStr;
#[cfg(not(feature = "dev"))]
use std::{
    fs,
    io::{self, BufRead},
    process,
};
#[cfg(not(feature = "dev"))]
use toml;

/// Returns a path to the directory storing application data
#[cfg(not(feature = "dev"))]
pub fn get_data_dir() -> PathBuf {
    let mut dir = dirs::home_dir().expect("Can't find home directory.");
    dir.push(".filite");
    dir
}

/// Returns a path to the configuration file
#[cfg(not(feature = "dev"))]
fn get_config_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("config.toml");
    path
}

/// Returns a path to the bearer token hash
#[cfg(not(feature = "dev"))]
pub fn get_password_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("passwd");
    path
}

/// Returns the BLAKE2b digest of the input string
pub fn hash<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    let mut hasher = Blake2b::new();
    hasher.input(input);
    hasher.result().to_vec()
}

/// Returns an environment variable and panic if it isn't found
#[cfg(feature = "dev")]
#[macro_export]
macro_rules! get_env {
    ($k:literal) => {
        std::env::var($k).expect(&format!("Can't find {} environment variable", $k));
    };
}

/// Returns a parsed environment variable and panic if it isn't found or is not parsable
#[cfg(feature = "dev")]
macro_rules! parse_env {
    ($k:literal) => {
        get_env!($k).parse().expect(&format!("Invalid {}", $k))
    };
}

/// Application configuration
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(not(feature = "dev"), serde(default))]
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

#[cfg(not(feature = "dev"))]
impl Default for Config {
    fn default() -> Self {
        let port = 8080;
        let database_url = {
            let mut path = get_data_dir();
            path.push("database.db");
            path.to_str()
                .expect("Can't convert database path to string")
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
    #[cfg(not(feature = "dev"))]
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
    #[cfg(not(feature = "dev"))]
    pub fn write_file(&self) -> Result<(), &'static str> {
        let path = get_config_path();
        let contents = toml::to_string(&self).expect("Can't serialize config.");
        match fs::write(&path, &contents) {
            Ok(_) => Ok(()),
            Err(_) => Err("Can't write config file."),
        }
    }

    /// Creates a config from environment variables
    #[cfg(feature = "dev")]
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
                    .expect("Can't convert cargo manifest dir to path");
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
        .expect("Can't create pool")
}

/// Initializes the logger
pub fn init_logger() {
    if cfg!(feature = "dev") && env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "actix_web=debug");
    } else if !cfg!(feature = "dev") {
        env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
}

/// Returns the logger middleware
pub fn logger_middleware() -> Logger {
    #[cfg(feature = "dev")]
    {
        dotenv::dotenv().ok();
        if let Ok(format) = env::var("LOG_FORMAT") {
            Logger::new(&format)
        } else {
            Logger::default()
        }
    }

    #[cfg(not(feature = "dev"))]
    {
        Logger::default()
    }
}

/// Performs the initial setup
#[cfg(not(feature = "dev"))]
pub fn init() -> Config {
    let data_dir = get_data_dir();
    if !data_dir.exists() {
        eprintln!("Generating config file...");
        fs::create_dir_all(&data_dir)
            .unwrap_or_else(|e| eprintln!("Can't create config directory: {}.", e));
        Config::default().write_file().unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });

        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        eprintln!("Enter the password to use:");
        let mut password = String::new();
        stdin.read_line(&mut password).unwrap_or_else(|e| {
            eprintln!("Can't read password: {}", e);
            process::exit(1);
        });
        password = password.replace("\r", "");
        password = password.replace("\n", "");
        let password_hash = hash(&password);
        let password_path = get_password_path();
        fs::write(&password_path, password_hash.as_slice()).unwrap_or_else(|e| {
            eprintln!("Can't write password: {}", e);
            process::exit(1);
        });

        let mut config_path = data_dir.clone();
        config_path.push("config.toml");
        eprintln!(
            "Almost ready. To get started, edit the config file at {} and restart.",
            &config_path.to_str().unwrap(),
        );
        process::exit(0);
    }

    Config::read_file().unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    })
}
