#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;

#[cfg_attr(not(feature = "dev"), macro_use)]
extern crate diesel_migrations;

#[cfg(feature = "dev")]
use crate::setup::Config;

use actix_web::{web, App, FromRequest, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use std::process;

pub mod models;
pub mod queries;
pub mod routes;
pub mod schema;
pub mod setup;

/// SQLite database connection pool
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[cfg(not(feature = "dev"))]
embed_migrations!("./migrations");

fn main() {
    let config = {
        #[cfg(feature = "dev")]
        {
            Config::debug()
        }

        #[cfg(not(feature = "dev"))]
        {
            setup::init()
        }
    };
    setup::init_logger();

    let pool = setup::create_pool(&config.database_url, config.pool_size);
    #[cfg(not(feature = "dev"))]
    embedded_migrations::run(&pool.get().unwrap()).unwrap_or_else(|e| {
        eprintln!("Can't prepare database: {}.", e);
        process::exit(1);
    });

    let port = config.port;
    let max_filesize = (config.max_filesize as f64 * 1.37) as usize;

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(pool.clone())
            .wrap(setup::logger_middleware())
            .service(web::resource("/config").route(web::get().to(routes::get_config)))
            .service(web::resource("/f").route(web::get().to_async(routes::files::gets)))
            .service(web::resource("/l").route(web::get().to_async(routes::links::gets)))
            .service(web::resource("/t").route(web::get().to_async(routes::texts::gets)))
            .route("/f/{id}", web::get().to_async(routes::files::get))
            .route("/l/{id}", web::get().to_async(routes::links::get))
            .route("/t/{id}", web::get().to_async(routes::texts::get))
            .service(
                web::resource("/f/{id}")
                    .data(web::Json::<routes::files::PutFile>::configure(|cfg| {
                        cfg.limit(max_filesize)
                    }))
                    .route(web::put().to_async(routes::files::put))
                    .route(web::delete().to_async(routes::files::delete)),
            )
            .service(
                web::resource("/l/{id}")
                    .route(web::put().to_async(routes::links::put))
                    .route(web::delete().to_async(routes::links::delete)),
            )
            .service(
                web::resource("/t/{id}")
                    .route(web::put().to_async(routes::texts::put))
                    .route(web::delete().to_async(routes::texts::delete)),
            )
    })
    .bind(&format!("localhost:{}", port))
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
