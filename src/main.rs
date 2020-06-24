#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
compile_error!("You need to select at least one database backend");

mod config;
mod db;
mod logger;
mod runtime;

use anyhow::Error;
use config::Config;
use db::pool::Pool;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// Configuration file to use
    ///
    /// If unspecified, will look for a filite.json
    /// file in the current working directory.
    #[structopt(short, long, name = "FILE")]
    config: Option<PathBuf>,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    /// Initialises the configuration file with default values
    InitConfig {
        /// File to write
        ///
        /// If unspecified, will write to a filite.json
        /// file in the current working directory.
        #[structopt(name = "FILE")]
        path: Option<PathBuf>,
    },
    /// Initialises the database tables
    InitDatabase {
        /// Database connection URL
        #[structopt(name = "URL")]
        url: String,
    },
}

fn main() -> Result<(), Error> {
    let args: Opt = Opt::from_args();
    match &args.command {
        Some(Command::InitConfig { path }) => {
            return init_config(match path {
                Some(_) => path.as_ref(),
                None => args.config.as_ref(),
            })
        }
        Some(Command::InitDatabase { url }) => return init_database(url),
        None => (),
    }

    let config = Config::read(args.config.unwrap_or_else(|| PathBuf::from("filite.json")))?;
    logger::init(&config.logger)?;

    let mut runtime = runtime::build(&config)?;
    runtime.block_on(run(config))?;

    Ok(())
}

async fn run(config: Config) -> Result<(), Error> {
    let _pool = Pool::build(&config).await?;
    Ok(())
}

fn init_config(path: Option<&PathBuf>) -> Result<(), Error> {
    config::Config::write(path.unwrap_or(&PathBuf::from("filite.json")))?;
    println!("Default config written");
    Ok(())
}

fn init_database(_url: &str) -> Result<(), Error> {
    Ok(())
}
