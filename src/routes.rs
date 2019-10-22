//! Actix route handlers

use crate::setup::{self, Config};
use actix_identity::Identity;
use actix_web::{error::BlockingError, web, HttpRequest, HttpResponse, Responder};
use base64;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel;
use serde::Serialize;

#[cfg(feature = "dev")]
use crate::get_env;
#[cfg(feature = "dev")]
use std::{fs, path::PathBuf};

/// Parses an ID
fn parse_id(id: &str) -> Result<i32, HttpResponse> {
    match i32::from_str_radix(id, 36) {
        Ok(id) => Ok(id),
        Err(_) => Err(HttpResponse::BadRequest().finish()),
    }
}

/// Checks for authentication
fn auth(identity: Identity, request: HttpRequest, token_hash: &[u8]) -> Result<(), HttpResponse> {
    if identity.identity().is_some() {
        return Ok(());
    }

    let header = match request.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(h) => h,
            Err(_) => return Err(HttpResponse::BadRequest().finish()),
        },
        None => {
            return Err(HttpResponse::Unauthorized()
                .header("WWW-Authenticate", "Bearer realm=\"filite\"")
                .finish())
        }
    };
    let token = header.replace("Bearer ", "");
    match setup::hash(&token).as_slice() == token_hash {
        true => Ok(()),
        false => Err(HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Bearer realm=\"filite\"")
            .finish()),
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
            request: HttpRequest,
            query: actix_web::web::Query<SelectQuery>,
            pool: actix_web::web::Data<Pool>,
            identity: actix_identity::Identity,
            token_hash: actix_web::web::Data<Vec<u8>>,
        ) -> impl futures::Future<Item = actix_web::HttpResponse, Error = actix_web::Error> {
            let filters = crate::queries::SelectFilters::from(query.into_inner());
            futures::future::result(crate::routes::auth(identity, request, &token_hash))
                .and_then(move |_| {
                    actix_web::web::block(move || crate::queries::$m::select(filters, pool)).then(
                        |result| match result {
                            Ok(x) => Ok(actix_web::HttpResponse::Ok().json(x)),
                            Err(_) => Err(actix_web::HttpResponse::InternalServerError().finish()),
                        },
                    )
                })
                .from_err()
        }
    };
}

/// DELETE an entry
macro_rules! delete {
    ($m:ident) => {
        pub fn delete(
            request: HttpRequest,
            path: actix_web::web::Path<String>,
            pool: actix_web::web::Data<Pool>,
            identity: actix_identity::Identity,
            token_hash: actix_web::web::Data<Vec<u8>>,
        ) -> impl futures::Future<Item = actix_web::HttpResponse, Error = actix_web::Error> {
            futures::future::result(crate::routes::auth(identity, request, &token_hash))
                .and_then(move |_| futures::future::result(crate::routes::parse_id(&path)))
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

#[cfg(feature = "dev")]
lazy_static! {
    static ref RESOURCES_DIR: PathBuf = {
        let mut ressources_dir = PathBuf::new();
        ressources_dir.push(get_env!("CARGO_MANIFEST_DIR"));
        ressources_dir.push("resources");
        ressources_dir.push("web");
        ressources_dir
    };
    static ref HTML_PATH: PathBuf = {
        let mut html_path = RESOURCES_DIR.clone();
        html_path.push("index.html");
        html_path
    };
    static ref JS_PATH: PathBuf = {
        let mut js_path = RESOURCES_DIR.clone();
        js_path.push("script.js");
        js_path
    };
    static ref CSS_PATH: PathBuf = {
        let mut css_path = RESOURCES_DIR.clone();
        css_path.push("style.css");
        css_path
    };
}

#[cfg(not(feature = "dev"))]
lazy_static! {
    static ref INDEX_CONTENTS: String = {
        let html = include_str!("../resources/web/index.html");
        let js = include_str!("../resources/web/script.js");
        let css = include_str!("../resources/web/style.css");

        html.replace("{{ js }}", js).replace("{{ css }}", css)
    };
}

/// Index page letting users upload via a UI
pub fn index(_identity: Identity) -> impl Responder {
    let contents = {
        #[cfg(feature = "dev")]
        {
            let html = fs::read_to_string(&*HTML_PATH).expect("Can't read index.html");
            let js = fs::read_to_string(&*JS_PATH).expect("Can't read script.js");
            let css = fs::read_to_string(&*CSS_PATH).expect("Can't read style.css");

            html.replace("{{ js }}", &js).replace("{{ css }}", &css)
        }
        #[cfg(not(feature = "dev"))]
        {
            INDEX_CONTENTS.clone()
        }
    };

    HttpResponse::Ok()
        .header("Content-Type", "text/html")
        .body(contents)
}

/// GET the config info
pub fn get_config(
    request: HttpRequest,
    config: web::Data<Config>,
    identity: Identity,
    token_hash: web::Data<Vec<u8>>,
) -> impl Responder {
    match auth(identity, request, &token_hash) {
        Ok(_) => HttpResponse::Ok().json(config.get_ref()),
        Err(response) => response,
    }
}

/// Login route
pub fn login(
    request: HttpRequest,
    identity: Identity,
    token_hash: web::Data<Vec<u8>>,
) -> impl Responder {
    if identity.identity().is_some() {
        return HttpResponse::Found().header("Location", "..").finish();
    }

    let header = match request.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(h) => h,
            Err(_) => return HttpResponse::BadRequest().finish(),
        },
        None => {
            return HttpResponse::Unauthorized()
                .header("WWW-Authenticate", "Basic realm=\"filite\"")
                .finish()
        }
    };
    let connection_string = header.replace("Basic ", "");
    let (user, token) = match base64::decode(&connection_string) {
        Ok(c) => {
            let credentials: Vec<Vec<u8>> =
                c.splitn(2, |b| b == &b':').map(|s| s.to_vec()).collect();
            match credentials.len() {
                2 => (credentials[0].clone(), credentials[1].clone()),
                _ => return HttpResponse::BadRequest().finish(),
            }
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match setup::hash(token).as_slice() == token_hash.as_slice() {
        true => match String::from_utf8(user.to_vec()) {
            Ok(u) => {
                identity.remember(u);
                HttpResponse::Found().header("Location", "..").finish()
            }
            Err(_) => HttpResponse::BadRequest().finish(),
        },
        false => HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Basic realm=\"filite\"")
            .finish(),
    }
}

/// Logout route
pub fn logout(identity: Identity) -> impl Responder {
    match identity.identity().is_some() {
        true => {
            identity.forget();
            HttpResponse::Found().header("Location", "..").finish()
        }
        false => HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Bearer realm=\"filite\"")
            .finish(),
    }
}

pub mod files {
    use crate::{
        queries::{self, SelectQuery},
        routes::{auth, match_find_error, parse_id},
        setup::Config,
        Pool,
    };
    use actix_files::NamedFile;
    use actix_identity::Identity;
    use actix_web::{error::BlockingError, http, web, Error, HttpRequest, HttpResponse};
    use chrono::Utc;
    use futures::{future, Future};
    use std::{fs, path::PathBuf};

    select!(files);

    /// GET a file entry and statically serve it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
    ) -> impl Future<Item = NamedFile, Error = Error> {
        future::result(parse_id(&path))
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
        request: HttpRequest,
        path: web::Path<String>,
        body: web::Json<PutFile>,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
        identity: Identity,
        token_hash: web::Data<Vec<u8>>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        future::result(auth(identity, request, &token_hash))
            .and_then(move |_| future::result(parse_id(&path)))
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
        routes::{
            auth, match_find_error, match_replace_result, parse_id, timestamp_to_last_modified,
        },
        Pool,
    };
    use actix_identity::Identity;
    use actix_web::{web, Error, HttpRequest, HttpResponse};
    use futures::{future, Future};

    select!(links);

    /// GET a link entry and redirect to it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        future::result(parse_id(&path))
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
        request: HttpRequest,
        path: web::Path<String>,
        body: web::Json<PutLink>,
        pool: web::Data<Pool>,
        identity: Identity,
        token_hash: web::Data<Vec<u8>>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        future::result(auth(identity, request, &token_hash))
            .and_then(move |_| future::result(parse_id(&path)))
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
        routes::{
            auth, match_find_error, match_replace_result, parse_id, timestamp_to_last_modified,
        },
        Pool,
    };
    use actix_identity::Identity;
    use actix_web::{web, Error, HttpRequest, HttpResponse};
    use futures::{future, Future};

    select!(texts);

    /// GET a text entry and display it
    pub fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        future::result(parse_id(&path))
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
        request: HttpRequest,
        path: web::Path<String>,
        body: web::Json<PutText>,
        pool: web::Data<Pool>,
        identity: Identity,
        token_hash: web::Data<Vec<u8>>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        future::result(auth(identity, request, &token_hash))
            .and_then(move |_| future::result(parse_id(&path)))
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
