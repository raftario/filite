use std::fmt::Display;
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::{Reply, Response},
};

#[derive(Debug, Copy, Clone)]
enum FiliteRejection {
    NotFound,
    Unauthorized,
    InternalServerError,
}
impl Reject for FiliteRejection {}
impl Reply for FiliteRejection {
    fn into_response(self) -> Response {
        match self {
            FiliteRejection::NotFound => {
                warp::reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response()
            }
            FiliteRejection::Unauthorized => warp::reply::with_status(
                warp::reply::with_header(
                    "Unauthorized",
                    "WWW-Authenticate",
                    r#"Basic realm="filite""#,
                ),
                StatusCode::UNAUTHORIZED,
            )
            .into_response(),
            FiliteRejection::InternalServerError => {
                warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        }
    }
}

#[inline]
pub fn unauthorized() -> Rejection {
    warp::reject::custom(FiliteRejection::Unauthorized)
}

pub trait TryExt<T> {
    fn or_404(self) -> Result<T, Rejection>;
    fn or_401(self) -> Result<T, Rejection>;
    fn or_500(self) -> Result<T, Rejection>;
}

impl<T, E: Display> TryExt<T> for Result<T, E> {
    fn or_404(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            warp::reject::custom(FiliteRejection::NotFound)
        })
    }

    fn or_401(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            warp::reject::custom(FiliteRejection::Unauthorized)
        })
    }

    fn or_500(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::error!("{}", e);
            warp::reject::custom(FiliteRejection::InternalServerError)
        })
    }
}

impl<T> TryExt<T> for Option<T> {
    fn or_404(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::NotFound))
    }

    fn or_401(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::Unauthorized))
    }

    fn or_500(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::InternalServerError))
    }
}

#[tracing::instrument(level = "debug")]
pub async fn handle_rejections(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
        Ok(FiliteRejection::NotFound)
    } else if let Some(err) = err.find::<FiliteRejection>() {
        Ok(*err)
    } else {
        Err(err)
    }
}
