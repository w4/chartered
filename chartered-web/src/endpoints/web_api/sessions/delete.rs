use axum::{extract, Json};
use chartered_db::{users::UserSession, ConnectionPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub async fn handle_delete(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Json(req): extract::Json<Request>,
) -> Result<Json<Response>, Error> {
    if UserSession::delete_by_uuid(db.clone(), req.uuid).await? {
        Ok(Json(Response { success: true }))
    } else {
        Err(Error::UnknownSession)
    }
}

#[derive(Deserialize)]
pub struct Request {
    uuid: chartered_db::uuid::Uuid,
}

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

#[derive(Serialize)]
pub struct ResponseSession {
    uuid: chartered_db::uuid::Uuid,
    expires_at: Option<chrono::NaiveDateTime>,
    user_agent: Option<String>,
    ip: Option<String>,
    ssh_key_fingerprint: Option<String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Unknown session")]
    UnknownSession,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            Self::Database(e) => e.status_code(),
            Self::UnknownSession => axum::http::StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);
