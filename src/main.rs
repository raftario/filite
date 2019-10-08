#[macro_use]
extern crate cfg_if;

use filite::setup::{self, Config};

use actix_web::{middleware, web, App, HttpServer, HttpResponse, Responder};
use std::process;

#[cfg(not(debug_assertions))]
use std::fs;

/// Performs the initial setup
#[cfg(not(debug_assertions))]
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

fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

    let pool = setup::create_pool(&config.database_url, config.pool_size);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
    })
    .bind(&format!("localhost:{}", config.port))
    .unwrap_or_else(|e| {
        eprintln!("Can't bind webserver to specified port: {}.", e);
        process::exit(1);
    })
    .run()
    .unwrap_or_else(|e| {
        eprintln!("Can't start webserver: {}.", e);
        process::exit(1);
    });
}
