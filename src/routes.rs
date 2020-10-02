use crate::{
    config::Config,
    db::{Filite, FiliteInner, User},
    reject::TryExt,
};
use askama::Template;
use bytes::Bytes;
use sled::Db;
use tokio::task;
use warp::{
    http::{StatusCode, Uri},
    reply::{Reply, Response},
    Filter, Rejection,
};

pub fn handler(
    config: &'static Config,
    db: &'static Db,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Copy + Send + Sync + 'static {
    let spectre = warp::path!("spectre.css").map(|| {
        warp::reply::with_header(
            include_str!("../static/spectre.min.css"),
            "Content-Type",
            "text/css",
        )
    });

    let index = warp::path::end()
        .and(crate::auth::auth(db, config))
        .and_then(index);

    let get = warp::path!(String)
        .and(warp::get())
        .and_then(move |id| get(id, db));

    let post_file = warp::path!("f")
        .and(warp::post())
        .and(crate::auth::auth(db, config))
        .and(warp::body::bytes())
        .and(warp::header::optional("Content-Type"))
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, data, mime, len| post_file(user, data, mime, len, db));
    let put_file = warp::path!("f" / String)
        .and(warp::put())
        .and(crate::auth::auth(db, config))
        .and(warp::body::bytes())
        .and(warp::header::optional("Content-Type"))
        .and_then(move |id, user, data, mime| put_file(id, user, data, mime, db));

    let post_link = warp::path!("l")
        .and(warp::post())
        .and(crate::auth::auth(db, config))
        .and(crate::util::body())
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, location, len| post_link(user, location, len, db));
    let put_link = warp::path!("l" / String)
        .and(warp::put())
        .and(crate::auth::auth(db, config))
        .and(crate::util::body())
        .and_then(move |id, user, location| put_link(id, user, location, db));

    let post_text = warp::path!("t")
        .and(warp::post())
        .and(crate::auth::auth(db, config))
        .and(crate::util::body())
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, data, len| post_text(user, data, len, db));
    let put_text = warp::path!("t" / String)
        .and(warp::put())
        .and(crate::auth::auth(db, config))
        .and(crate::util::body())
        .and_then(move |id, user, data| put_text(id, user, data, db));

    spectre
        .or(index)
        .or(get)
        .or(post_file)
        .or(put_file)
        .or(post_link)
        .or(put_link)
        .or(post_text)
        .or(put_text)
        .recover(crate::reject::handle_rejections)
}

#[tracing::instrument(level = "debug")]
async fn index(user: User) -> Result<impl Reply, Rejection> {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct Template<'a> {
        title: &'a str,
        user: &'a str,
    }

    let template = Template {
        title: "filite",
        user: &user.id,
    };
    Ok(warp::reply::html(template.render().or_500()?))
}

#[tracing::instrument(level = "debug", skip(db))]
async fn get(id: String, db: &Db) -> Result<impl Reply, Rejection> {
    impl Reply for Filite {
        fn into_response(self) -> Response {
            match self.inner {
                FiliteInner::File { data, mime } => {
                    warp::reply::with_header(data, "Content-Type", mime).into_response()
                }
                FiliteInner::Link { location } => {
                    warp::redirect::temporary(Uri::from_maybe_shared(location).unwrap_or_default())
                        .into_response()
                }
                FiliteInner::Text { data } => data.into_response(),
            }
        }
    }

    let filite = task::block_in_place(|| crate::db::get(&id, db))
        .or_500()?
        .or_404()?;
    Ok(filite)
}

#[tracing::instrument(level = "debug", skip(db))]
async fn post_file(
    user: User,
    data: Bytes,
    mime: Option<String>,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = task::block_in_place(|| crate::db::random_id(len.unwrap_or(8), db)).or_500()?;
    put_file(id, user, data, mime, db).await
}

#[tracing::instrument(level = "debug", skip(db))]
async fn put_file(
    id: String,
    user: User,
    data: Bytes,
    mime: Option<String>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    task::block_in_place(|| {
        crate::db::insert_file(
            &id,
            user.id,
            data.to_vec(),
            mime.unwrap_or_else(|| "application/octet-stream".to_owned()),
            db,
        )
    })
    .or_500()?
    .or_409()?;
    Ok(warp::reply::with_status(id, StatusCode::CREATED))
}

#[tracing::instrument(level = "debug", skip(db))]
async fn post_link(
    user: User,
    location: Uri,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = task::block_in_place(|| crate::db::random_id(len.unwrap_or(8), db)).or_500()?;
    put_link(id, user, location, db).await
}

#[tracing::instrument(level = "debug", skip(db))]
async fn put_link(id: String, user: User, location: Uri, db: &Db) -> Result<impl Reply, Rejection> {
    task::block_in_place(|| crate::db::insert_link(&id, user.id, location.to_string(), db))
        .or_500()?
        .or_409()?;
    Ok(warp::reply::with_status(id, StatusCode::CREATED))
}

#[tracing::instrument(level = "debug", skip(db))]
async fn post_text(
    user: User,
    data: String,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = task::block_in_place(|| crate::db::random_id(len.unwrap_or(8), db)).or_500()?;
    put_text(id, user, data, db).await
}

#[tracing::instrument(level = "debug", skip(db))]
async fn put_text(id: String, user: User, data: String, db: &Db) -> Result<impl Reply, Rejection> {
    task::block_in_place(|| crate::db::insert_text(&id, user.id, data, db))
        .or_500()?
        .or_409()?;
    Ok(warp::reply::with_status(id, StatusCode::CREATED))
}
