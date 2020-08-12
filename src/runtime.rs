use crate::config::Config;
use anyhow::Error;
use tokio::runtime::{Builder, Runtime};

#[tracing::instrument(level = "debug")]
pub fn build(config: &Config) -> Result<Runtime, Error> {
    let mut builder = Builder::new();
    builder.threaded_scheduler().enable_all();

    let config = &config.threads;
    if let Some(ct) = config.core_threads {
        builder.core_threads(ct);
    }
    if let Some(mt) = config.max_threads {
        builder.max_threads(mt);
    }

    Ok(builder.build()?)
}
