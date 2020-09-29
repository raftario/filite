use crate::config::RuntimeConfig;
use anyhow::Error;
use tokio::runtime::{Builder, Runtime};

#[tracing::instrument(level = "debug")]
pub fn build(config: &RuntimeConfig) -> Result<Runtime, Error> {
    let mut builder = Builder::new();
    builder.threaded_scheduler().enable_all();

    if let Some(ct) = config.core_threads {
        builder.core_threads(ct);
    }
    if let Some(mt) = config.max_threads {
        builder.max_threads(mt);
    }

    Ok(builder.build()?)
}
