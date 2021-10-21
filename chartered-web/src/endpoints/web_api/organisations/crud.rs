//! Allows users to create whole organisations. This endpoint currently isn't limited to any
//! specific users so all users can create an organisation and add people to it.

use axum::{extract, Json};
use chartered_db::{organisations::Organisation, users::User, ConnectionPool};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

use crate::endpoints::ErrorResponse;

pub async fn handle_put(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    Organisation::create(db, req.name, req.description, req.public, user.id).await?;

    Ok(Json(ErrorResponse { error: None }))
}

#[derive(Deserialize)]
pub struct PutRequest {
    name: String,
    description: String,
    public: bool,
}

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
