#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate serde;

use filite::queries;
use filite::setup::{self, Config};
use filite::{Pool, SelectRange};

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

/// Query string for SELECT queries
#[derive(Deserialize)]
struct SelectQuery {
    /// Left creation bounder timestamp
    cf: Option<i32>,
    /// Right creation bounder timestamp
    ct: Option<i32>,
    /// Left update bounder timestamp
    uf: Option<i32>,
    /// Right update bounder timestamp
    ut: Option<i32>,
    /// Query size limit
    limit: Option<i64>,
    /// Whether to sort the results in ascending order
    asc: Option<bool>,
    /// Whether to sort the results by creation date
    created: Option<bool>,
}

/// Filters for SELECT queries
struct SelectFilters {
    /// Creation and update date and time ranges
    range: SelectRange,
    /// Query size limit
    limit: Option<i64>,
    /// Whether to sort the results in ascending order
    order_asc: bool,
    /// Whether to sort the results by creation date
    order_created: bool,
}

impl From<SelectQuery> for SelectFilters {
    fn from(query: SelectQuery) -> Self {
        SelectFilters {
            range: SelectRange {
                created: (query.cf, query.ct),
                updated: (query.uf, query.ut),
            },
            limit: query.limit,
            order_asc: query.asc.unwrap_or(false),
            order_created: query.created.unwrap_or(false),
        }
    }
}

/// GET multiple file entries
fn get_files(
    query: web::Query<SelectQuery>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let filters = SelectFilters::from(query.into_inner());
    web::block(move || {
        queries::files::select(
            filters.range,
            filters.limit,
            filters.order_asc,
            filters.order_created,
            pool,
        )
    })
    .then(|result| match result {
        Ok(files) => Ok(HttpResponse::Ok().json(files)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
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

    let port = config.port;

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .service(web::resource("/f").route(web::get().to_async(get_files)))
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
