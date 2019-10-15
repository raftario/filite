use crate::setup::Config;

use actix_web::error::BlockingError;
use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::num;

/// GET multiple entries
macro_rules! select {
    ($m:ident) => {
        pub fn gets(
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

/// Parses a base 36 ID
#[inline(always)]
fn id_from_b36(s: &str) -> Result<i32, num::ParseIntError> {
    i32::from_str_radix(s, 36)
}

/// Parses an ID and errors if it fails
macro_rules! parse_id {
    ($s:expr) => {
        match id_from_b36($s) {
            Ok(id) => id,
            Err(_) => return Either::B(future::err(HttpResponse::BadRequest().finish().into())),
        };
    };
}

/// Match result from REPLACE queries
macro_rules! put_then {
    ($f:expr) => {
        $f.then(|result| match result {
            Ok(x) => Ok(HttpResponse::Created().json(x)),
            Err(_) => Err(HttpResponse::InternalServerError().finish().into()),
        })
    };
}

/// Handles error from single GET queries using find
fn find_error<T>(error: BlockingError<diesel::result::Error>) -> Result<T, actix_web::Error> {
    match error {
        BlockingError::Error(e) => match e {
            diesel::result::Error::NotFound => Err(HttpResponse::NotFound().finish().into()),
            _ => Err(HttpResponse::InternalServerError().finish().into()),
        },
        BlockingError::Canceled => Err(HttpResponse::InternalServerError().finish().into()),
    }
}

/// DELETE an entry
macro_rules! delete {
    ($m:ident) => {
        pub fn delete(
            path: web::Path<String>,
            pool: web::Data<Pool>,
        ) -> impl Future<Item = HttpResponse, Error = Error> {
            let id = parse_id!(&path);
            Either::A(web::block(move || queries::$m::delete(id, pool)).then(
                |result| match result {
                    Ok(()) => Ok(HttpResponse::NoContent().finish()),
                    Err(e) => find_error(e),
                },
            ))
        }
    };
}

/// Formats a timestamp to the "Last-Modified" header format
fn timestamp_to_last_modified(timestamp: i32) -> String {
    let datetime =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp as i64, 0), Utc);
    datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// GET the config info
pub fn get_config(config: web::Data<Config>) -> impl Responder {
    HttpResponse::Ok().json(config.get_ref())
}

pub mod files {
    use crate::handlers::{find_error, id_from_b36};
    use crate::queries::{self, SelectFilters, SelectQuery};
    use crate::setup::Config;
    use crate::Pool;

    use actix_files::NamedFile;
    use actix_web::error::BlockingError;
    use actix_web::{http, web, Error, HttpResponse};
    use chrono::{Datelike, Utc};
    use futures::future::{self, Either};
    use futures::Future;
    use std::fs;
    use std::path::PathBuf;

    select!(files);

    /// GET a file entry and statically serve it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
    ) -> impl Future<Item = NamedFile, Error = Error> {
        let id = parse_id!(&path);
        let files_dir = config.files_dir.clone();
        Either::A(
            web::block(move || queries::files::find(id, pool)).then(|result| match result {
                Ok(file) => {
                    let mut path = files_dir;
                    path.push(file.filepath);
                    match NamedFile::open(&path) {
                        Ok(nf) => Ok(nf),
                        Err(_) => Err(HttpResponse::NotFound().finish().into()),
                    }
                }
                Err(e) => find_error(e),
            }),
        )
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
        let id = parse_id!(&path);
        Either::A(
            web::block(move || {
                let mut path = config.files_dir.clone();
                let mut relative_path = PathBuf::new();

                let current_time = Utc::now();
                let current_date = current_time.date().naive_utc();
                path.push(&format!("{:04}", current_date.year()));
                relative_path.push(&format!("{:04}", current_date.year()));
                path.push(&format!("{:02}", current_date.month()));
                relative_path.push(&format!("{:02}", current_date.month()));
                path.push(&format!("{:02}", current_date.day()));
                relative_path.push(&format!("{:02}", current_date.day()));

                if fs::create_dir_all(&path).is_err() {
                    return Err(http::StatusCode::from_u16(500).unwrap());
                }

                let mut filename = body.filename.clone();
                let timestamp = format!("{:x}.", current_time.timestamp());
                filename.insert_str(0, &timestamp);
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
                    BlockingError::Error(sc) => Err(HttpResponse::new(sc).into()),
                    BlockingError::Canceled => {
                        Err(HttpResponse::InternalServerError().finish().into())
                    }
                },
            }),
        )
    }

    delete!(files);
}

pub mod links {
    use crate::handlers::{find_error, id_from_b36, timestamp_to_last_modified};
    use crate::queries::{self, SelectFilters, SelectQuery};
    use crate::Pool;

    use actix_web::{web, Error, HttpResponse};
    use futures::future::{self, Either};
    use futures::Future;

    select!(links);

    /// GET a link entry and redirect to it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        let id = parse_id!(&path);
        Either::A(
            web::block(move || queries::links::find(id, pool)).then(|result| match result {
                Ok(link) => Ok(HttpResponse::Found()
                    .header("Location", link.forward)
                    .header("Last-Modified", timestamp_to_last_modified(link.created))
                    .finish()),
                Err(e) => find_error(e),
            }),
        )
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
        let id = parse_id!(&path);
        Either::A(put_then!(web::block(move || queries::links::replace(
            id,
            &body.forward,
            pool
        ))))
    }

    delete!(links);
}

pub mod texts {
    use crate::handlers::{find_error, id_from_b36, timestamp_to_last_modified};
    use crate::queries::{self, SelectFilters, SelectQuery};
    use crate::Pool;

    use actix_web::{web, Error, HttpResponse};
    use futures::future::{self, Either};
    use futures::Future;

    select!(texts);

    /// GET a text entry and display it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        let id = parse_id!(&path);
        Either::A(
            web::block(move || queries::texts::find(id, pool)).then(|result| match result {
                Ok(text) => Ok(HttpResponse::Ok()
                    .header("Last-Modified", timestamp_to_last_modified(text.created))
                    .body(text.contents)),
                Err(e) => find_error(e),
            }),
        )
    }

    /// Request body when PUTting texts
    #[derive(Deserialize)]
    pub struct PutText {
        pub contents: String,
    }

    /// PUT a new text entry
    pub fn put(
        (path, body): (web::Path<String>, web::Json<PutText>),
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        let id = parse_id!(&path);
        Either::A(put_then!(web::block(move || queries::texts::replace(
            id,
            &body.contents,
            pool
        ))))
    }

    delete!(texts);
}
