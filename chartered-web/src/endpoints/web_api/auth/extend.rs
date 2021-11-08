//! Extends a user's session. Called on a loop from the web UI to keep
//! the user from logging out during periods of idleness from the API's
//! perspective.

use axum::{extract, Json};
use chartered_db::users::UserSession;
use chartered_db::ConnectionPool;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle(
    extract::Extension(session): extract::Extension<Arc<UserSession>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
) -> Result<Json<ExtendResponse>, Error> {
    let expires = chrono::Utc::now() + chrono::Duration::hours(1);

    if session.extend(db, expires.naive_utc()).await? {
        Ok(Json(ExtendResponse { expires }))
    } else {
        Err(Error::InvalidSession)
    }
}

#[derive(Debug, Serialize)]
pub struct ExtendResponse {
    expires: chrono::DateTime<chrono::Utc>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Invalid session")]
    InvalidSession,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::InvalidSession => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);
