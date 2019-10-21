//! Actix route handlers

use crate::setup::Config;
use actix_web::{error::BlockingError, web, HttpResponse, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel;
use futures::future::{self, FutureResult};
use serde::Serialize;

/// Parses an ID
fn parse_id(id: &str) -> FutureResult<i32, HttpResponse> {
    match i32::from_str_radix(id, 36) {
        Ok(id) => future::ok(id),
        Err(_) => future::err(HttpResponse::BadRequest().finish()),
    }
}

/// Match result from REPLACE queries
#[inline(always)]
fn match_replace_result<T: Serialize>(
    result: Result<T, BlockingError<diesel::result::Error>>,
) -> Result<HttpResponse, HttpResponse> {
    match result {
        Ok(x) => Ok(HttpResponse::Created().json(x)),
        Err(_) => Err(HttpResponse::InternalServerError().finish()),
    }
}

/// Handles error from single GET queries using find
#[inline(always)]
fn match_find_error<T>(error: BlockingError<diesel::result::Error>) -> Result<T, HttpResponse> {
    match error {
        BlockingError::Error(e) => match e {
            diesel::result::Error::NotFound => Err(HttpResponse::NotFound().finish()),
            _ => Err(HttpResponse::InternalServerError().finish()),
        },
        BlockingError::Canceled => Err(HttpResponse::InternalServerError().finish()),
    }
}

/// Formats a timestamp to the "Last-Modified" header format
fn timestamp_to_last_modified(timestamp: i32) -> String {
    let datetime =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp as i64, 0), Utc);
    datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// GET multiple entries
macro_rules! select {
    ($m:ident) => {
        pub fn gets(
            query: actix_web::web::Query<SelectQuery>,
            pool: actix_web::web::Data<Pool>,
        ) -> impl futures::Future<Item = actix_web::HttpResponse, Error = actix_web::Error> {
            let filters = crate::queries::SelectFilters::from(query.into_inner());
            actix_web::web::block(move || crate::queries::$m::select(filters, pool))
                .then(|result| match result {
                    Ok(x) => Ok(actix_web::HttpResponse::Ok().json(x)),
                    Err(_) => Err(actix_web::HttpResponse::InternalServerError().finish()),
                })
                .from_err()
        }
    };
}

/// DELETE an entry
macro_rules! delete {
    ($m:ident) => {
        pub fn delete(
            path: actix_web::web::Path<String>,
            pool: actix_web::web::Data<Pool>,
        ) -> impl futures::Future<Item = actix_web::HttpResponse, Error = actix_web::Error> {
            crate::routes::parse_id(&path)
                .and_then(move |id| {
                    actix_web::web::block(move || crate::queries::$m::delete(id, pool)).then(
                        |result| match result {
                            Ok(()) => Ok(actix_web::HttpResponse::NoContent().finish()),
                            Err(e) => crate::routes::match_find_error(e),
                        },
                    )
                })
                .from_err()
        }
    };
}

/// GET the config info
pub fn get_config(config: web::Data<Config>) -> impl Responder {
    HttpResponse::Ok().json(config.get_ref())
}

pub mod files {
    use crate::{
        queries::{self, SelectQuery},
        routes::{match_find_error, parse_id},
        setup::Config,
        Pool,
    };
    use actix_files::NamedFile;
    use actix_web::{error::BlockingError, http, web, Error, HttpResponse};
    use chrono::Utc;
    use futures::Future;
    use std::{fs, path::PathBuf};

    select!(files);

    /// GET a file entry and statically serve it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
    ) -> impl Future<Item = NamedFile, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || queries::files::find(id, pool)).then(
                    move |result| match result {
                        Ok(file) => {
                            let mut path = config.files_dir.clone();
                            path.push(file.filepath);
                            match NamedFile::open(&path) {
                                Ok(nf) => Ok(nf),
                                Err(_) => Err(HttpResponse::NotFound().finish()),
                            }
                        }
                        Err(e) => match_find_error(e),
                    },
                )
            })
            .from_err()
    }

    /// Request body when PUTting files
    #[derive(Deserialize)]
    pub struct PutFile {
        pub base64: String,
        pub filename: String,
    }

    /// PUT a new file entry
    pub fn put(
        (path, body): (web::Path<String>, web::Json<PutFile>),
        config: web::Data<Config>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || {
                let mut path = config.files_dir.clone();
                let mut relative_path = PathBuf::new();
                if fs::create_dir_all(&path).is_err() {
                    return Err(http::StatusCode::from_u16(500).unwrap());
                }

                let mut filename = body.filename.clone();
                filename = format!("{:x}.{}", Utc::now().timestamp(), filename);
                path.push(&filename);
                relative_path.push(&filename);

                let relative_path = match relative_path.to_str() {
                    Some(rp) => rp,
                    None => return Err(http::StatusCode::from_u16(500).unwrap()),
                };

                let contents = match base64::decode(&body.base64) {
                    Ok(contents) => contents,
                    Err(_) => return Err(http::StatusCode::from_u16(400).unwrap()),
                };
                if fs::write(&path, contents).is_err() {
                    return Err(http::StatusCode::from_u16(500).unwrap());
                }

                match queries::files::replace(id, relative_path, pool) {
                    Ok(file) => Ok(file),
                    Err(_) => Err(http::StatusCode::from_u16(500).unwrap()),
                }
            })
            .then(|result| match result {
                Ok(file) => Ok(HttpResponse::Created().json(file)),
                Err(e) => match e {
                    BlockingError::Error(sc) => Err(HttpResponse::new(sc)),
                    BlockingError::Canceled => {
                        Err(HttpResponse::InternalServerError().finish())
                    }
                },
            })
            })
            .from_err()
    }

    delete!(files);
}

pub mod links {
    use crate::{
        queries::{self, SelectQuery},
        routes::{match_find_error, match_replace_result, parse_id, timestamp_to_last_modified},
        Pool,
    };
    use actix_web::{web, Error, HttpResponse};
    use futures::Future;

    select!(links);

    /// GET a link entry and redirect to it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || queries::links::find(id, pool)).then(|result| match result {
                    Ok(link) => Ok(HttpResponse::Found()
                        .header("Location", link.forward)
                        .header("Last-Modified", timestamp_to_last_modified(link.created))
                        .finish()),
                    Err(e) => match_find_error(e),
                })
            })
            .from_err()
    }

    /// Request body when PUTting links
    #[derive(Deserialize)]
    pub struct PutLink {
        pub forward: String,
    }

    /// PUT a new link entry
    pub fn put(
        (path, body): (web::Path<String>, web::Json<PutLink>),
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || queries::links::replace(id, &body.forward, pool))
                    .then(|result| match_replace_result(result))
            })
            .from_err()
    }

    delete!(links);
}

pub mod texts {
    use crate::{
        queries::{self, SelectQuery},
        routes::{match_find_error, match_replace_result, parse_id, timestamp_to_last_modified},
        Pool,
    };
    use actix_web::{web, Error, HttpResponse};
    use futures::Future;

    select!(texts);

    /// GET a text entry and display it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || queries::texts::find(id, pool)).then(|result| match result {
                    Ok(text) => Ok(HttpResponse::Ok()
                        .header("Last-Modified", timestamp_to_last_modified(text.created))
                        .body(text.contents)),
                    Err(e) => match_find_error(e),
                })
            })
            .from_err()
    }

    /// Request body when PUTting texts
    #[derive(Deserialize)]
    pub struct PutText {
        pub contents: String,
        pub highlight: bool,
    }

    /// PUT a new text entry
    pub fn put(
        (path, body): (web::Path<String>, web::Json<PutText>),
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        parse_id(&path)
            .and_then(move |id| {
                web::block(move || {
                    queries::texts::replace(id, &body.contents, body.highlight, pool)
                })
                .then(|result| match_replace_result(result))
            })
            .from_err()
    }

    delete!(texts);
}
