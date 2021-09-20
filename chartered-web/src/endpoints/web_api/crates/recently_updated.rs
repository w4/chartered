use axum::{extract, Json};
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("{0}")]
    CrateFetch(#[from] crate::models::crates::CrateFetchError),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CrateFetch(e) => e.status_code(),
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crates_with_versions = Crate::list_recently_updated(db, user.id).await?;

    Ok(Json(Response {
        versions: crates_with_versions
            .into_iter()
            .map(|(crate_, version)| ResponseVersion {
                name: crate_.name,
                version: version.version,
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct Response {
    versions: Vec<ResponseVersion>,
}

#[derive(Serialize)]
pub struct ResponseVersion {
    name: String,
    version: String,
}
