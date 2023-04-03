use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

pub mod auth;

#[macro_export]
macro_rules! response {
    ($item:expr) => {{
        match $item {
            Ok(item) => Response::ok(item),
            Err(e) => Response::err(300, e.to_string()),
        }
    }};
    ($item:expr,$typ:ty) => {{
        match $item {
            Ok(item) => Response::ok(<$typ>::from(item)),
            Err(e) => Response::err(300, e.to_string()),
        }
    }};
}

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn new(code: i32, msg: String, data: Option<T>) -> Self {
        Self { code, msg, data }
    }
    pub fn ok(data: T) -> Self {
        Self::new(200, "OK".to_string(), Some(data))
    }
    pub fn err(code: i32, msg: String) -> Self {
        Self::new(code, msg, None)
    }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
