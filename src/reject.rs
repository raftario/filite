use std::fmt::{Debug, Display};
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::{Reply, Response},
};

#[derive(Debug, Clone)]
enum FiliteRejection {
    NotFound,
    Unauthorized,
    InternalServerError,
    Conflict,

    Custom(String, StatusCode),
}
impl Reject for FiliteRejection {}
impl Reply for FiliteRejection {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => {
                warp::reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response()
            }
            Self::Unauthorized => warp::reply::with_status(
                warp::reply::with_header(
                    "Unauthorized",
                    "WWW-Authenticate",
                    r#"Basic realm="filite""#,
                ),
                StatusCode::UNAUTHORIZED,
            )
            .into_response(),
            Self::InternalServerError => {
                warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
            Self::Conflict => {
                warp::reply::with_status("Conflict", StatusCode::CONFLICT).into_response()
            }
            Self::Custom(reply, status) => warp::reply::with_status(reply, status).into_response(),
        }
    }
}

#[inline]
pub fn unauthorized() -> Rejection {
    warp::reject::custom(FiliteRejection::Unauthorized)
}

#[inline]
pub fn custom<T: ToString>(reply: T, status: StatusCode) -> Rejection {
    warp::reject::custom(FiliteRejection::Custom(reply.to_string(), status))
}

pub trait TryExt<T> {
    fn or_404(self) -> Result<T, Rejection>;
    fn or_401(self) -> Result<T, Rejection>;
    fn or_500(self) -> Result<T, Rejection>;
    fn or_409(self) -> Result<T, Rejection>;
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

    fn or_409(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            warp::reject::custom(FiliteRejection::Conflict)
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

    fn or_409(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::Conflict))
    }
}

#[tracing::instrument(level = "debug")]
pub async fn handle_rejections(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
        Ok(FiliteRejection::NotFound)
    } else if let Some(err) = err.find::<FiliteRejection>() {
        Ok(err.clone())
    } else {
        Err(err)
    }
}
