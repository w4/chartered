use crate::models::crates::get_crate_with_permissions;
use axum::{extract, Json};
use chartered_db::{
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
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

#[derive(Serialize)]
pub struct GetResponse {
    users: Vec<GetResponseUser>,
}

#[derive(Serialize)]
pub struct GetResponseUser {
    // cargo spec says this should be an unsigned 32-bit integer
    // uuid: chartered_db::uuid::Uuid,
    login: String,
    name: Option<String>,
}

pub async fn handle_get(
    extract::Path((_session_key, name)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_ = get_crate_with_permissions(db.clone(), user, name, &[Permission::VISIBLE]).await?;

    let users = crate_
        .owners(db)
        .await?
        .into_iter()
        .map(|user| GetResponseUser {
            // uuid: user.uuid.0,
            login: user.username,
            name: None,
        })
        .collect();

    Ok(Json(GetResponse { users }))
}
