//! Logs a user out of the session used to send this request and deletes it from the
//! database.

use axum::{extract, Json};
use chartered_db::users::UserSession;
use chartered_db::ConnectionPool;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle(
    extract::Extension(session): extract::Extension<Arc<UserSession>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
) -> Result<Json<LogoutResponse>, Error> {
    session.delete(db).await?;

    Ok(Json(LogoutResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    success: bool,
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
