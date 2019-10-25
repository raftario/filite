#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

#[cfg_attr(not(feature = "dev"), macro_use)]
extern crate diesel_migrations;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web, App, FromRequest, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    sqlite::SqliteConnection,
};
use std::process;

#[cfg(feature = "dev")]
use crate::setup::Config;
#[cfg(feature = "dev")]
use dotenv;
#[cfg(not(feature = "dev"))]
use std::fs;

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
    {
        embedded_migrations::run(&pool.get().unwrap()).unwrap_or_else(|e| {
            eprintln!("Can't prepare database: {}.", e);
            process::exit(1);
        });
    }

    let password_hash = {
        #[cfg(feature = "dev")]
        {
            dotenv::dotenv().ok();
            let password = get_env!("PASSWD");
            setup::hash(&password)
        }
        #[cfg(not(feature = "dev"))]
        {
            let password_path = setup::get_password_path();
            fs::read(&password_path).unwrap_or_else(|e| {
                eprintln!("Can't read password hash from disk: {}.", e);
                process::exit(1);
            })
        }
    };

    let port = config.port;
    let max_filesize = (config.max_filesize as f64 * 1.37) as usize;

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(config.clone())
            .data(password_hash.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("filite-auth-cookie")
                    .secure(false),
            ))
            .wrap(setup::logger_middleware())
            .route("/", web::get().to(routes::index))
            .route("/logout", web::get().to(routes::logout))
            .route("/config", web::get().to(routes::get_config))
            .route("/f", web::get().to_async(routes::files::gets))
            .route("/l", web::get().to_async(routes::links::gets))
            .route("/t", web::get().to_async(routes::texts::gets))
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
