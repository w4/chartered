#[derive(serde::Serialize)]
pub struct ErrorResponse {
    error: &'static str,
}

macro_rules! define_error {
    ($($kind:ident$(($inner_name:ident: $inner:ty))? => $status:ident / $public_text:expr,)*) => {
        #[derive(thiserror::Error, Debug)]
        pub enum Error {
            $($kind$((#[from] $inner))?),*
        }

        /// a (web-safe) explanation of the error, this shouldn't reveal internal details that
        /// may be sensitive
        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$kind$(($inner_name))? => f.write_str($public_text)),*
                }
            }
        }

        impl axum::response::IntoResponse for Error {
            type Body = axum::body::Full<axum::body::Bytes>;
            type BodyError = <Self::Body as axum::body::HttpBody>::Error;

            fn into_response(self) -> axum::http::Response<Self::Body> {
                let (status, body) = match self {
                    $(Self::$kind$(($inner_name))? => (
                        axum::http::StatusCode::$status,
                        serde_json::to_vec(&crate::endpoints::ErrorResponse { error: $public_text }).unwrap(),
                    )),*
                };

                let mut res = axum::http::Response::new(axum::body::Full::from(body));
                *res.status_mut() = status;
                res.headers_mut().insert(axum::http::header::CONTENT_TYPE, axum::http::header::HeaderValue::from_static("application/json"));
                res
            }
        }
    };
}

pub mod cargo_api;
