use std::borrow::Cow;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: Option<Cow<'static, str>>,
}

macro_rules! define_error_response {
    ($error:ty) => {
        impl crate::middleware::logging::GenericError for $error {}

        impl axum::response::IntoResponse for $error {
            fn into_response(self) -> axum::response::Response {
                let response = crate::endpoints::ErrorResponse {
                    error: Some(self.to_string().into()),
                };

                let mut response = (self.status_code(), axum::Json(response)).into_response();
                response
                    .extensions_mut()
                    .insert::<Box<dyn crate::middleware::logging::GenericError>>(Box::new(self));
                response
            }
        }
    };
}

pub mod cargo_api;
pub mod web_api;
