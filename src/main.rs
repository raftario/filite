#[macro_use]
extern crate cfg_if;

use filite::queries::{self, SelectFilters, SelectQuery};
use filite::setup::{self, Config};
use filite::Pool;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use futures::Future;
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

/// GET multiple entries
macro_rules! select {
    ($n:ident, $m:ident) => {
        fn $n(
            query: web::Query<SelectQuery>,
            pool: web::Data<Pool>,
        ) -> impl Future<Item = HttpResponse, Error = Error> {
            let filters = SelectFilters::from(query.into_inner());
            web::block(move || queries::$m::select(filters, pool)).then(|result| match result {
                Ok(x) => Ok(HttpResponse::Ok().json(x)),
                Err(_) => Ok(HttpResponse::InternalServerError().into()),
            })
        }
    };
}

select!(get_files, files);
select!(get_links, links);
select!(get_texts, texts);

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

    let port = config.port;

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .service(web::resource("/f").route(web::get().to_async(get_files)))
            .service(web::resource("/l").route(web::get().to_async(get_links)))
            .service(web::resource("/t").route(web::get().to_async(get_texts)))
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
