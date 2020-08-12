mod auth;
mod config;
mod db;
mod routes;
mod runtime;
mod utils;

use anyhow::Error;
use config::Config;
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Reply};

#[derive(StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// Configuration file to use
    #[structopt(
        short,
        long,
        name = "FILE",
        env = "FILITE_CONFIG",
        default_value = "filite.json"
    )]
    config: String,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    /// Initialises the configuration file with default values
    Init,
}

fn main() -> Result<(), Error> {
    let args: Opt = Opt::from_args();

    if let Some(Command::Init) = &args.command {
        config::write(&args.config)?;
        println!("Default config written");
        return Ok(());
    }

    let config = config::read(&args.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let mut runtime = runtime::build(&config)?;
    runtime.block_on(run(config))?;

    Ok(())
}

async fn run(config: &'static Config) -> Result<(), Error> {
    let pool = db::pool::build(&config).await?;
    Ok(())
}

async fn serve(
    filter: impl Filter<Extract = (impl Reply,)> + Send + Sync + Clone + 'static,
    config: &Config,
) {
    match &config.tls {
        Some(tls_config) => {
            warp::serve(filter)
                .tls()
                .cert_path(&tls_config.cert)
                .key_path(&tls_config.key)
                .run(([127, 0, 0, 1], config.port))
                .await
        }
        None => warp::serve(filter).run(([127, 0, 0, 1], config.port)).await,
    }
}
