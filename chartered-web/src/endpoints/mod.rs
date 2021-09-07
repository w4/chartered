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
            type Body = axum::body::Body;
            type BodyError = <Self::Body as axum::body::HttpBody>::Error;

            fn into_response(self) -> axum::http::Response<Self::Body> {
                log::error!("Failed to handle request: {:?}", self);

                let (status, body) = match self {
                    $(Self::$kind$(($inner_name))? => (
                        axum::http::StatusCode::$status,
                        $public_text.into(),
                    )),*
                };

                axum::http::Response::builder()
                    .status(status)
                    .body(body)
                    .unwrap()
            }
        }
    };
}

pub mod cargo_api;
