use crate::config::LoggerConfig;
use anyhow::Error;
use log::{self, Level, LevelFilter, Log, Metadata, Record};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Stderr, Stdout, Write},
    sync::Mutex,
};

pub fn init(config: &LoggerConfig) -> Result<(), Error> {
    let logger = Logger {
        console: (
            config.console.0,
            Mutex::new(io::stderr()),
            Mutex::new(io::stdout()),
        ),
        file: match &config.file {
            Some(fc) => Some((
                fc.level.0,
                Mutex::new(BufWriter::new(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&fc.path)?,
                )),
            )),
            None => None,
        },
    };
    log::set_boxed_logger(Box::new(logger))?;
    Ok(())
}

struct Logger {
    console: (LevelFilter, Mutex<Stderr>, Mutex<Stdout>),
    file: Option<(LevelFilter, Mutex<BufWriter<File>>)>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.console.0
            || self
                .file
                .as_ref()
                .map_or(false, |(lf, ..)| metadata.level() <= *lf)
    }

    fn log(&self, record: &Record) {
        let target = record.target();
        let level = record.level();
        let args = record.args();

        if let Some((lf, bw)) = &self.file {
            if level <= *lf {
                bw.lock()
                    .unwrap()
                    .write_fmt(format_args!("[{}]::[{}] {}\n", target, level, args))
                    .ok();
            }
        }

        if level <= self.console.0 {
            if level <= Level::Warn {
                self.console
                    .1
                    .lock()
                    .unwrap()
                    .write_fmt(format_args!("[{}]::[{}] {}\n", target, level, args))
                    .ok();
            } else {
                self.console
                    .2
                    .lock()
                    .unwrap()
                    .write_fmt(format_args!("[{}]::[{}] {}\n", target, level, args))
                    .ok();
            }
        }
    }

    fn flush(&self) {
        self.console.1.lock().unwrap().flush().ok();
        self.console.2.lock().unwrap().flush().ok();
        if let Some((_, bw)) = &self.file {
            bw.lock().unwrap().flush().ok();
        }
    }
}
