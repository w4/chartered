use chartered_db::{users::User, ConnectionPool};

use axum::{extract, Json};
use chartered_db::uuid::Uuid;
use chrono::{DateTime, TimeZone, Utc};
use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::endpoints::ErrorResponse;

#[derive(Serialize)]
pub struct GetResponse {
    keys: Vec<GetResponseKey>,
}

#[derive(Serialize)]
pub struct GetResponseKey {
    uuid: Uuid,
    name: String,
    fingerprint: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

pub async fn handle_get(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let keys = user
        .list_ssh_keys(db)
        .await?
        .into_iter()
        .map(|key| GetResponseKey {
            uuid: key.uuid.0,
            fingerprint: key.fingerprint().unwrap_or_else(|e| {
                warn!("Failed to parse key with id {}: {}", key.id, e);
                "INVALID".to_string()
            }),
            name: key.name,
            created_at: Utc.from_local_datetime(&key.created_at).unwrap(),
            last_used_at: key
                .last_used_at
                .and_then(|v| Utc.from_local_datetime(&v).single()),
        })
        .collect();

    Ok(Json(GetResponse { keys }))
}

#[derive(Deserialize)]
pub struct PutRequest {
    key: String,
}

pub async fn handle_put(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    match user.insert_ssh_key(db, &req.key).await {
        Ok(()) => Ok(Json(ErrorResponse { error: None })),
        Err(e @ chartered_db::Error::KeyParse(_)) => Err(Error::KeyParse(e)),
        Err(e) => Err(Error::Database(e)),
    }
}

pub async fn handle_delete(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Path((_session_key, ssh_key_id)): extract::Path<(String, Uuid)>,
) -> Result<Json<ErrorResponse>, Error> {
    let deleted = user.delete_user_ssh_key_by_uuid(db, ssh_key_id).await?;

    if deleted {
        Ok(Json(ErrorResponse { error: None }))
    } else {
        Err(Error::NonExistentKey)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to parse SSH key: {0}")]
    KeyParse(chartered_db::Error),
    #[error("The key given does not exist")]
    NonExistentKey,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::KeyParse(_) | Self::NonExistentKey => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);
