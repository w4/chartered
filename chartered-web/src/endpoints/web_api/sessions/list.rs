use axum::{extract, Json};
use chartered_db::users::UserSession;
use chartered_db::{users::User, ConnectionPool};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle_get(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let sessions = UserSession::list(db.clone(), user.id).await?;

    Ok(Json(Response {
        sessions: sessions
            .into_iter()
            .map(|(session, ssh_key)| ResponseSession {
                expires_at: session.expires_at,
                user_agent: session.user_agent,
                ip: session.ip,
                ssh_key_fingerprint: ssh_key
                    .map(|v| v.fingerprint().unwrap_or_else(|_| "INVALID".to_string())),
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct Response {
    sessions: Vec<ResponseSession>,
}

#[derive(Serialize)]
pub struct ResponseSession {
    expires_at: Option<chrono::NaiveDateTime>,
    user_agent: Option<String>,
    ip: Option<String>,
    ssh_key_fingerprint: Option<String>,
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
