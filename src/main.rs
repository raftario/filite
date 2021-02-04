#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

#[cfg_attr(not(feature = "dev"), macro_use)]
extern crate diesel_migrations;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web, App, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    sqlite::SqliteConnection,
};
use std::process;

pub mod globals;
pub mod models;
pub mod queries;
pub mod routes;
pub mod schema;
pub mod setup;

/// SQLite database connection pool
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[cfg(not(feature = "dev"))]
embed_migrations!();

use globals::{CONFIG, KEY};

#[actix_rt::main]
async fn main() {
    setup::init_logger();

    #[cfg(not(feature = "dev"))]
    {
        embedded_migrations::run(&globals::POOL.get().unwrap()).unwrap_or_else(|e| {
            eprintln!("Can't prepare database: {}", e);
            process::exit(1);
        });
    }

    let port = CONFIG.port;
    println!("Listening on port {}", port);

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(KEY)
                    .name("filite-auth-cookie")
                    .secure(true),
            ))
            .wrap(setup::logger_middleware())
            .route("/", web::get().to(routes::index))
            .route("/highlight.min.js", web::get().to(routes::js))
            .route("/spectre-icons.min.css", web::get().to(routes::icon))
            .route("/spectre.min.css", web::get().to(routes::css))
            .route("/logout", web::get().to(routes::logout))
            .route("/config", web::get().to(routes::get_config))
            .route("/id/{id}", web::get().to(routes::id_to_str))
            .service(
                web::resource("/f")
                    .route(web::get().to(routes::files::select))
                    .route(web::post().to(routes::files::post)),
            )
            .service(
                web::resource("/l")
                    .route(web::get().to(routes::links::select))
                    .route(web::post().to(routes::links::post)),
            )
            .service(
                web::resource("/t")
                    .route(web::get().to(routes::texts::select))
                    .route(web::post().to(routes::texts::post)),
            )
            .service(
                web::resource("/f/{id}")
                    .route(web::get().to(routes::files::get))
                    .route(web::put().to(routes::files::put))
                    .route(web::delete().to(routes::files::delete)),
            )
            .service(
                web::resource("/l/{id}")
                    .route(web::get().to(routes::links::get))
                    .route(web::put().to(routes::links::put))
                    .route(web::delete().to(routes::links::delete)),
            )
            .service(
                web::resource("/t/{id}")
                    .route(web::get().to(routes::texts::get))
                    .route(web::put().to(routes::texts::put))
                    .route(web::delete().to(routes::texts::delete)),
            )
    })
    .bind(&format!("localhost:{}", port))
    .unwrap_or_else(|e| {
        eprintln!("Can't bind webserver to specified port: {}", e);
        process::exit(1);
    })
    .run()
    .await
    .unwrap_or_else(|e| {
        eprintln!("Can't start webserver: {}", e);
        process::exit(1);
    });
}
