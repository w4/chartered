use chartered_db::{users::User, ConnectionPool};

use axum::{extract, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Serialize)]
pub struct GetResponse {
    keys: Vec<GetResponseKey>,
}

#[derive(Serialize)]
pub struct GetResponseKey {
    id: i32, // TODO: this should be a UUID so we don't leak incremental IDs
    fingerprint: String,
}

pub async fn handle_get(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let keys = user
        .list_ssh_keys(db)
        .await?
        .into_iter()
        .map(|(id, fingerprint)| GetResponseKey { id, fingerprint })
        .collect();

    Ok(Json(GetResponse { keys }))
}

#[derive(Deserialize)]
pub struct PutRequest {
    key: String,
}

#[derive(Serialize)]
pub struct PutResponse {
    error: bool,
}

pub async fn handle_put(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutRequest>,
) -> Result<Json<PutResponse>, Error> {
    match user.insert_ssh_key(db, &req.key).await {
        Ok(()) => Ok(Json(PutResponse { error: false })),
        Err(e @ chartered_db::Error::KeyParse(_)) => Err(Error::KeyParse(e)),
        Err(e) => Err(Error::Database(e)),
    }
}

#[derive(Serialize)]
pub struct DeleteResponse {
    error: bool,
}

pub async fn handle_delete(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Path((_session_key, ssh_key_id)): extract::Path<(String, i32)>,
) -> Result<Json<DeleteResponse>, Error> {
    let deleted = user.delete_user_ssh_key_by_id(db, ssh_key_id).await?;

    if deleted {
        Ok(Json(DeleteResponse { error: false }))
    } else {
        Err(Error::NonExistentKey)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
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
