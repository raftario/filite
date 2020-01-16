//! Actix route handlers

use crate::setup::{self, Config};
use actix_identity::Identity;
use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse, Responder};
use base64;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel;
use serde::Serialize;
use std::convert::Infallible;

#[cfg(feature = "dev")]
use crate::get_env;
#[cfg(feature = "dev")]
use std::{fs, path::PathBuf};

/// Parses an ID
fn parse_id(id: &str) -> Result<i32, HttpResponse> {
    match i32::from_str_radix(id, 36) {
        Ok(id) => Ok(id),
        Err(_) => Err(HttpResponse::BadRequest().body("Invalid ID")),
    }
}

lazy_static! {
    static ref EMPTY_HASH: Vec<u8> = setup::hash("");
}

/// Authenticates a user
async fn auth(
    identity: Identity,
    request: HttpRequest,
    password_hash: &[u8],
) -> Result<(), HttpResponse> {
    if identity.identity().is_some() {
        return Ok(());
    }

    if password_hash == (&*EMPTY_HASH).as_slice() {
        identity.remember("guest".into());
        return Ok(());
    }

    let header = match request.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(h) => h,
            Err(_) => return Err(HttpResponse::BadRequest().body("Invalid Authorization header")),
        },
        None => {
            return Err(HttpResponse::Unauthorized()
                .header("WWW-Authenticate", "Basic realm=\"filite\"")
                .body("Unauthorized"))
        }
    };
    let connection_string = header.replace("Basic ", "");
    let (user, password) = match base64::decode(&connection_string) {
        Ok(c) => {
            let credentials: Vec<Vec<u8>> = c
                .splitn(2, |b| b == &b':')
                .map(|s| s.to_vec())
                .collect::<Vec<Vec<u8>>>();
            match credentials.len() {
                2 => (credentials[0].clone(), credentials[1].clone()),
                _ => return Err(HttpResponse::BadRequest().body("Invalid Authorization header")),
            }
        }
        Err(_) => return Err(HttpResponse::BadRequest().body("Invalid Authorization header")),
    };

    let infallible_hash = move || -> Result<Vec<u8>, Infallible> { Ok(setup::hash(password)) };
    if web::block(infallible_hash).await.unwrap().as_slice() == password_hash {
        match String::from_utf8(user.to_vec()) {
            Ok(u) => {
                identity.remember(u);
                Ok(())
            }
            Err(_) => Err(HttpResponse::BadRequest().body("Invalid Authorization header")),
        }
    } else {
        Err(HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Basic realm=\"filite\"")
            .body("Unauthorized"))
    }
}

/// Match result from REPLACE queries
fn match_replace_result<T: Serialize>(
    result: Result<T, BlockingError<diesel::result::Error>>,
    id: i32,
) -> Result<HttpResponse, Error> {
    match result {
        Ok(_) => Ok(HttpResponse::Created().body(format!("{}", radix_fmt::radix_36(id)))),
        Err(_) => Err(HttpResponse::InternalServerError()
            .body("Internal server error")
            .into()),
    }
}

/// Handles error from single GET queries using find
fn match_find_error<T>(error: BlockingError<diesel::result::Error>) -> Result<T, Error> {
    match error {
        BlockingError::Error(e) => match e {
            diesel::result::Error::NotFound => {
                Err(HttpResponse::NotFound().body("Not found").into())
            }
            _ => Err(HttpResponse::InternalServerError()
                .body("Internal server error")
                .into()),
        },
        BlockingError::Canceled => Err(HttpResponse::InternalServerError()
            .body("Internal server error")
            .into()),
    }
}

/// Formats a timestamp to the "Last-Modified" header format
fn timestamp_to_last_modified(timestamp: i32) -> String {
    let datetime =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(i64::from(timestamp), 0), Utc);
    datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Escapes text to be inserted in a HTML element
fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

/// GET multiple entries
macro_rules! select {
    ($m:ident) => {
        pub async fn select(
            request: HttpRequest,
            query: actix_web::web::Query<SelectQuery>,
            pool: actix_web::web::Data<Pool>,
            identity: actix_identity::Identity,
            password_hash: actix_web::web::Data<Vec<u8>>,
        ) -> Result<actix_web::HttpResponse, actix_web::Error> {
            crate::routes::auth(identity, request, &password_hash).await?;

            let filters = crate::queries::SelectFilters::from(query.into_inner());
            match actix_web::web::block(move || crate::queries::$m::select(filters, pool)).await {
                Ok(x) => Ok(actix_web::HttpResponse::Ok().json(x)),
                Err(_) => Err(actix_web::HttpResponse::InternalServerError()
                    .body("Internal server error")
                    .into()),
            }
        }
    };
}

/// DELETE an entry
macro_rules! delete {
    ($m:ident) => {
        pub async fn delete(
            request: HttpRequest,
            path: actix_web::web::Path<String>,
            pool: actix_web::web::Data<Pool>,
            identity: actix_identity::Identity,
            password_hash: actix_web::web::Data<Vec<u8>>,
        ) -> Result<actix_web::HttpResponse, actix_web::Error> {
            crate::routes::auth(identity, request, &password_hash).await?;

            let id = crate::routes::parse_id(&path)?;
            match actix_web::web::block(move || crate::queries::$m::delete(id, pool)).await {
                Ok(()) => Ok(actix_web::HttpResponse::NoContent().body("Deleted")),
                Err(e) => crate::routes::match_find_error(e),
            }
        }
    };
}

/// Verify if an entry exists
macro_rules! random_id {
    ($m:ident) => {
        use rand::distributions::Distribution;

        pub async fn random_id(pool: &actix_web::web::Data<Pool>) -> Result<i32, actix_web::Error> {
            let mut rng = rand::thread_rng();
            let distribution = rand::distributions::Uniform::from(0..i32::max_value());
            loop {
                let id = distribution.sample(&mut rng);
                let pool = pool.clone();
                match actix_web::web::block(move || crate::queries::$m::find(id, pool)).await {
                    Ok(_) => continue,
                    Err(e) => match e {
                        actix_web::error::BlockingError::Error(e) => match e {
                            diesel::result::Error::NotFound => return Ok(id),
                            _ => {
                                return Err(actix_web::HttpResponse::InternalServerError()
                                    .body("Internal server error")
                                    .into())
                            }
                        },
                        actix_web::error::BlockingError::Canceled => {
                            return Err(actix_web::HttpResponse::InternalServerError()
                                .body("Internal server error")
                                .into())
                        }
                    },
                }
            }
        }
    };
}

#[cfg(feature = "dev")]
lazy_static! {
    static ref INDEX_PATH: PathBuf = {
        let mut index_path = PathBuf::new();
        index_path.push(get_env!("CARGO_MANIFEST_DIR"));
        index_path.push("resources");
        index_path.push("index.html");
        index_path
    };
}

#[cfg(not(feature = "dev"))]
static INDEX_CONTENTS: &str = include_str!("../resources/index.html");

static HIGHLIGHT_CONTENTS: &str = include_str!("../resources/highlight.html");
const HIGHLIGHT_LANGUAGE: &str = r#"<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.15.10/languages/{{ language }}.min.js"></script>"#;

/// Index page letting users upload via a UI
pub async fn index(
    request: HttpRequest,
    identity: Identity,
    password_hash: web::Data<Vec<u8>>,
) -> impl Responder {
    if let Err(response) = auth(identity, request, &password_hash).await {
        return response;
    }

    let contents = {
        #[cfg(feature = "dev")]
        {
            fs::read_to_string(&*INDEX_PATH).expect("Can't read index.html")
        }
        #[cfg(not(feature = "dev"))]
        {
            INDEX_CONTENTS.to_owned()
        }
    };

    HttpResponse::Ok()
        .header("Content-Type", "text/html")
        .body(contents)
}

/// GET the config info
pub async fn get_config(
    request: HttpRequest,
    config: web::Data<Config>,
    identity: Identity,
    password_hash: web::Data<Vec<u8>>,
) -> impl Responder {
    match auth(identity, request, &password_hash).await {
        Ok(_) => HttpResponse::Ok().json(config.get_ref()),
        Err(response) => response,
    }
}

/// Logout route
pub async fn logout(identity: Identity) -> impl Responder {
    if identity.identity().is_some() {
        identity.forget();
        HttpResponse::Ok().body("Logged out")
    } else {
        HttpResponse::Unauthorized()
            .header("WWW-Authenticate", "Basic realm=\"filite\"")
            .body("Unauthorized")
    }
}

pub mod files {
    use crate::routes::match_replace_result;
    use crate::{
        queries::{self, SelectQuery},
        routes::{auth, match_find_error, parse_id},
        setup::Config,
        Pool,
    };
    use actix_files::NamedFile;
    use actix_identity::Identity;
    use actix_multipart::Multipart;
    use actix_web::{web, Error, HttpRequest, HttpResponse};
    use chrono::Utc;
    use futures::StreamExt;
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    select!(files);
    delete!(files);
    random_id!(files);

    /// GET a file entry and statically serve it
    pub async fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
    ) -> Result<NamedFile, Error> {
        let id = parse_id(&path)?;
        match web::block(move || queries::files::find(id, pool)).await {
            Ok(file) => {
                let mut path = config.files_dir.clone();
                path.push(file.filepath);
                match NamedFile::open(&path) {
                    Ok(nf) => Ok(nf),
                    Err(_) => Err(HttpResponse::NotFound().body("Not found").into()),
                }
            }
            Err(e) => match_find_error(e),
        }
    }

    /// Common code for PUT and POST routes
    async fn put_post(
        id: i32,
        mut body: Multipart,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
    ) -> Result<HttpResponse, Error> {
        let mut path = config.files_dir.clone();
        let mut relative_path = PathBuf::new();
        let dir_path = path.clone();
        if web::block(move || fs::create_dir_all(dir_path))
            .await
            .is_err()
        {
            return Err(HttpResponse::InternalServerError()
                .body("Internal server error")
                .into());
        }

        let mut field = match body.next().await {
            Some(f) => f?,
            None => {
                return Err(HttpResponse::BadRequest()
                    .body("Empty multipart body")
                    .into())
            }
        };
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => {
                return Err(HttpResponse::BadRequest()
                    .body("Missing content disposition")
                    .into())
            }
        };
        let filename = match content_disposition.get_filename() {
            Some(n) => n,
            None => return Err(HttpResponse::BadRequest().body("Missing filename").into()),
        };
        let filename = format!("{:x}.{}", Utc::now().timestamp(), filename);
        path.push(&filename);
        relative_path.push(&filename);
        let relative_path = match path.to_str() {
            Some(rp) => rp.to_owned(),
            None => {
                return Err(HttpResponse::InternalServerError()
                    .body("Internal server error")
                    .into())
            }
        };

        let mut f = match web::block(move || File::create(&path)).await {
            Ok(f) => f,
            Err(_) => {
                return Err(HttpResponse::InternalServerError()
                    .body("Internal server error")
                    .into())
            }
        };
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(c) => c,
                Err(_) => {
                    return Err(HttpResponse::BadRequest()
                        .body("Invalid multipart data")
                        .into())
                }
            };

            f = match web::block(move || match f.write_all(&data) {
                Ok(_) => Ok(f),
                Err(_) => Err(()),
            })
            .await
            {
                Ok(f) => f,
                Err(_) => {
                    return Err(HttpResponse::InternalServerError()
                        .body("Internal server error")
                        .into())
                }
            };
        }

        match_replace_result(
            web::block(move || queries::files::replace(id, &relative_path, pool)).await,
            id,
        )
    }

    /// PUT a new file entry
    pub async fn put(
        request: HttpRequest,
        path: web::Path<String>,
        body: Multipart,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = parse_id(&path)?;
        put_post(id, body, pool, config).await
    }

    /// POST a new file entry using a multipart body
    pub async fn post(
        request: HttpRequest,
        body: Multipart,
        pool: web::Data<Pool>,
        config: web::Data<Config>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = random_id(&pool).await?;
        put_post(id, body, pool, config).await
    }
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

    select!(links);
    delete!(links);
    random_id!(links);

    /// GET a link entry and redirect to it
    pub async fn get(
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let id = parse_id(&path)?;
        match web::block(move || queries::links::find(id, pool)).await {
            Ok(link) => Ok(HttpResponse::Found()
                .header("Location", link.forward)
                .header("Last-Modified", timestamp_to_last_modified(link.created))
                .finish()),
            Err(e) => match_find_error(e),
        }
    }

    /// Request body when PUTting links
    #[derive(Deserialize)]
    pub struct PutPostLink {
        pub forward: String,
    }

    /// PUT a new link entry
    pub async fn put(
        request: HttpRequest,
        path: web::Path<String>,
        body: web::Json<PutPostLink>,
        pool: web::Data<Pool>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = parse_id(&path)?;
        match_replace_result(
            web::block(move || queries::links::replace(id, &body.forward, pool)).await,
            id,
        )
    }

    /// POST a new link entry
    pub async fn post(
        request: HttpRequest,
        body: web::Json<PutPostLink>,
        pool: web::Data<Pool>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = random_id(&pool).await?;
        match_replace_result(
            web::block(move || queries::links::replace(id, &body.forward, pool)).await,
            id,
        )
    }
}

pub mod texts {
    use crate::routes::escape_html;
    use crate::{
        queries::{self, SelectQuery},
        routes::{
            auth, match_find_error, match_replace_result, parse_id, timestamp_to_last_modified,
        },
        Pool,
    };
    use crate::{
        routes::{HIGHLIGHT_CONTENTS, HIGHLIGHT_LANGUAGE},
        setup::Config,
    };
    use actix_identity::Identity;
    use actix_web::{web, Error, HttpRequest, HttpResponse};

    select!(texts);
    delete!(texts);
    random_id!(texts);

    /// GET a text entry and display it
    pub async fn get(
        config: web::Data<Config>,
        path: web::Path<String>,
        pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let id = parse_id(&path)?;
        match web::block(move || queries::texts::find(id, pool)).await {
            Ok(text) => {
                let last_modified = timestamp_to_last_modified(text.created);
                if text.highlight {
                    let languages: Vec<String> = config
                        .highlight
                        .languages
                        .iter()
                        .map(|l| HIGHLIGHT_LANGUAGE.replace("{{ language }}", l))
                        .collect();
                    let languages = languages.join("\n");
                    let contents = HIGHLIGHT_CONTENTS
                        .replace("{{ title }}", &path)
                        .replace("{{ theme }}", &config.highlight.theme)
                        .replace("{{ contents }}", &escape_html(&text.contents))
                        .replace("{{ languages }}", &languages);

                    Ok(HttpResponse::Ok()
                        .header("Last-Modified", last_modified)
                        .header("Content-Type", "text/html")
                        .body(contents))
                } else {
                    Ok(HttpResponse::Ok()
                        .header("Last-Modified", last_modified)
                        .body(text.contents))
                }
            }
            Err(e) => match_find_error(e),
        }
    }

    /// Request body when PUTting texts
    #[derive(Deserialize)]
    pub struct PutPostText {
        pub contents: String,
        pub highlight: bool,
    }

    /// PUT a new text entry
    pub async fn put(
        request: HttpRequest,
        path: web::Path<String>,
        body: web::Json<PutPostText>,
        pool: web::Data<Pool>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = parse_id(&path)?;
        match_replace_result(
            web::block(move || queries::texts::replace(id, &body.contents, body.highlight, pool))
                .await,
            id,
        )
    }

    /// POST a new text entry
    pub async fn post(
        request: HttpRequest,
        body: web::Json<PutPostText>,
        pool: web::Data<Pool>,
        identity: Identity,
        password_hash: web::Data<Vec<u8>>,
    ) -> Result<HttpResponse, Error> {
        auth(identity, request, &password_hash).await?;
        let id = random_id(&pool).await?;
        match_replace_result(
            web::block(move || queries::texts::replace(id, &body.contents, body.highlight, pool))
                .await,
            id,
        )
    }
}
