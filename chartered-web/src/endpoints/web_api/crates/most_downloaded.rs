use axum::{extract, Json};
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            Self::Database(e) => e.status_code(),
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crates_with_versions = Crate::list_most_downloaded(db, user.id).await?;

    Ok(Json(Response {
        crates: crates_with_versions
            .into_iter()
            .map(|(crate_, organisation)| ResponseCrate {
                name: crate_.name,
                downloads: crate_.downloads,
                organisation: organisation.name,
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct Response {
    crates: Vec<ResponseCrate>,
}

#[derive(Serialize)]
pub struct ResponseCrate {
    name: String,
    downloads: i32,
    organisation: String,
}
