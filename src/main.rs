#[macro_use]
extern crate cfg_if;

use filite::queries::{self, SelectFilters, SelectQuery};
use filite::setup::{self, Config};
use filite::Pool;

use actix_files::NamedFile;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
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
                Err(_) => Err(HttpResponse::InternalServerError().finish().into()),
            })
        }
    };
}

select!(get_files, files);
select!(get_links, links);
select!(get_texts, texts);

/// Returns a generic Not Found error response
#[inline(always)]
fn not_found() -> Error {
    HttpResponse::NotFound().finish().into()
}

/// Parses a base 36 ID
#[inline(always)]
fn parse_id(s: &str) -> i32 {
    i32::from_str_radix(s, 36).unwrap_or(-1)
}

/// Formats a timestamp to the "Last-Modified" header format
fn timestamp_to_last_modified(timestamp: i32) -> String {
    let datetime =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp as i64, 0), Utc);
    datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// GET a file entry and statically serve it
fn get_file(
    path: web::Path<String>,
    pool: web::Data<Pool>,
    config: web::Data<Config>,
) -> impl Future<Item = NamedFile, Error = Error> {
    let id = parse_id(&path);
    let files_dir = config.files_dir.clone();
    web::block(move || queries::files::find(id, pool)).then(|result| match result {
        Ok(file) => {
            let mut path = files_dir;
            path.push(file.filepath);
            match NamedFile::open(&path) {
                Ok(nf) => Ok(nf),
                Err(_) => Err(HttpResponse::NotFound().finish().into()),
            }
        }
        Err(_) => Err(not_found()),
    })
}

/// GET a link entry and redirect to it
fn get_link(
    path: web::Path<String>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = parse_id(&path);
    web::block(move || queries::links::find(id, pool)).then(|result| match result {
        Ok(link) => Ok(HttpResponse::Found()
            .header("Location", link.forward)
            .header("Last-Modified", timestamp_to_last_modified(link.updated))
            .finish()),
        Err(_) => Err(not_found()),
    })
}

/// GET a text entry and display it
fn get_text(
    path: web::Path<String>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = parse_id(&path);
    web::block(move || queries::texts::find(id, pool)).then(|result| match result {
        Ok(text) => Ok(HttpResponse::Ok()
            .header("Last-Modified", timestamp_to_last_modified(text.updated))
            .body(text.contents)),
        Err(_) => Err(not_found()),
    })
}

/// GET the config info
fn get_config(config: web::Data<Config>) -> impl Responder {
    HttpResponse::Ok().json(config.get_ref())
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

    let port = config.port;

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(pool.clone())
            .wrap(setup::logger_middleware())
            .route("/", web::get().to(index))
            .service(web::resource("/config").route(web::get().to(get_config)))
            .service(web::resource("/f").route(web::get().to_async(get_files)))
            .service(web::resource("/l").route(web::get().to_async(get_links)))
            .service(web::resource("/t").route(web::get().to_async(get_texts)))
            .route("/f/{id}", web::get().to_async(get_file))
            .route("/l/{id}", web::get().to_async(get_link))
            .route("/t/{id}", web::get().to_async(get_text))
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
