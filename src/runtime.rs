use crate::config::Config;
use anyhow::Error;
use log::debug;
use tokio::runtime::{Builder, Runtime};

#[cfg_attr(not(feature = "threaded"), allow(unused_variables))]
pub fn build(config: &Config) -> Result<Runtime, Error> {
    let mut builder = Builder::new();
    builder.basic_scheduler().enable_io();

    #[cfg(feature = "threaded")]
    {
        builder.threaded_scheduler();

        let config = &config.threads;
        if let Some(ct) = config.core_threads {
            builder.core_threads(ct);
        }
        if let Some(mt) = config.max_threads {
            builder.max_threads(mt);
        }
    }

    debug!("Building Tokio runtime");
    Ok(builder.build()?)
}
