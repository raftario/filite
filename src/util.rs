use bytes::Bytes;
use std::str::FromStr;
use warp::{Filter, Rejection};

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
            Ok(s) => match T::from_str(s) {
                Ok(v) => Ok(v),
                Err(e) => Err(crate::reject::bad_request(e)),
            },
            Err(e) => Err(crate::reject::bad_request(e)),
        }
    })
}
