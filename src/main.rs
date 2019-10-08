#[macro_use]
extern crate cfg_if;

use filite::setup::{self, Config};

use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use std::fs;
use std::process;

/// Performs the initial setup
fn init() -> Config {
    let data_dir = setup::get_data_dir();
    if !data_dir.exists() {
        eprintln!("Creating config file...");
        fs::create_dir_all(&data_dir)
            .unwrap_or_else(|e| eprintln!("Can't create config directory: {}.", e));
        Config::default().write_file().unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        eprintln!(
            "To get started, edit the config file at {:?} and restart.",
            &data_dir
        );
        process::exit(0);
    }

    Config::read_file().unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    })
}

fn main() {
    let config = {
        cfg_if! {
            if #[cfg(debug_assertions)] {
                Config::debug()
            } else {
                init()
            }
        }
    };
    setup::init_logger();
}
