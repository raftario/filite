use std::fmt::{Debug, Display};
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::{Reply, Response},
};

#[derive(Debug, Clone)]
enum FiliteRejection {
    BadRequest(String),
    Unauthorized(String),
    NotFound,
    Conflict,
    InternalServerError,
}
impl Reject for FiliteRejection {}
impl Reply for FiliteRejection {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(reply) => {
                warp::reply::with_status(reply, StatusCode::BAD_REQUEST).into_response()
            }
            Self::Unauthorized(reply) => warp::reply::with_status(
                warp::reply::with_header(reply, "WWW-Authenticate", r#"Basic realm="filite""#),
                StatusCode::UNAUTHORIZED,
            )
            .into_response(),
            Self::NotFound => {
                warp::reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response()
            }
            Self::Conflict => {
                warp::reply::with_status("Conflict", StatusCode::CONFLICT).into_response()
            }
            Self::InternalServerError => {
                warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        }
    }
}

pub fn bad_request(reply: impl ToString) -> Rejection {
    warp::reject::custom(FiliteRejection::BadRequest(reply.to_string()))
}
pub fn unauthorized(reply: impl ToString) -> Rejection {
    warp::reject::custom(FiliteRejection::Unauthorized(reply.to_string()))
}

pub trait TryExt<T> {
    fn or_404(self) -> Result<T, Rejection>;
    fn or_409(self) -> Result<T, Rejection>;

    fn or_500(self) -> Result<T, Rejection>;

    fn or_bad_request(self, reply: impl ToString) -> Result<T, Rejection>;
    fn or_unauthorized(self, reply: impl ToString) -> Result<T, Rejection>;
}

impl<T, E: Display> TryExt<T> for Result<T, E> {
    fn or_404(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            warp::reject::custom(FiliteRejection::NotFound)
        })
    }
    fn or_409(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            warp::reject::custom(FiliteRejection::Conflict)
        })
    }

    fn or_500(self) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::error!("{}", e);
            warp::reject::custom(FiliteRejection::InternalServerError)
        })
    }

    fn or_bad_request(self, reply: impl ToString) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            bad_request(reply)
        })
    }
    fn or_unauthorized(self, reply: impl ToString) -> Result<T, Rejection> {
        self.map_err(|e| {
            tracing::info!("{}", e);
            unauthorized(reply)
        })
    }
}

impl<T> TryExt<T> for Option<T> {
    fn or_404(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::NotFound))
    }
    fn or_409(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::Conflict))
    }

    fn or_500(self) -> Result<T, Rejection> {
        self.ok_or_else(|| warp::reject::custom(FiliteRejection::InternalServerError))
    }

    fn or_bad_request(self, reply: impl ToString) -> Result<T, Rejection> {
        self.ok_or_else(move || bad_request(reply))
    }
    fn or_unauthorized(self, reply: impl ToString) -> Result<T, Rejection> {
        self.ok_or_else(move || unauthorized(reply))
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
