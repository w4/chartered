use axum::{extract, Json};
use chartered_db::{organisations::Organisation, users::User, ConnectionPool};
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

pub async fn handle_get(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let organisations = Organisation::list(db.clone(), user.id).await?;

    Ok(Json(Response {
        organisations: organisations
            .into_iter()
            .map(|v| ResponseOrganisation {
                name: v.name,
                description: v.description,
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct Response {
    organisations: Vec<ResponseOrganisation>,
}

#[derive(Serialize)]
pub struct ResponseOrganisation {
    name: String,
    description: String,
}
