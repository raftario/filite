use bytes::Bytes;
use std::str::FromStr;
use warp::{http::StatusCode, Filter, Rejection};

pub trait DefaultExt {
    fn is_default(&self) -> bool;
}
impl<T: Default + PartialEq> DefaultExt for T {
    fn is_default(&self) -> bool {
        self.eq(&Default::default())
    }
}

pub fn body<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy + Send + Sync + 'static
where
    T: FromStr,
    T::Err: ToString,
{
    warp::body::bytes().and_then(|b: Bytes| async move {
        match std::str::from_utf8(&b) {
            Ok(s) => match s.parse() {
                Ok(v) => Ok(v),
                Err(e) => Err(crate::reject::custom(e, StatusCode::BAD_REQUEST)),
            },
            Err(e) => Err(crate::reject::custom(e, StatusCode::BAD_REQUEST)),
        }
    })
}
