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
pub struct Response {
    ok: bool,
}

pub async fn handle_yank(
    extract::Path((_session_key, name, version)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_ = get_crate_with_permissions(
        db.clone(),
        user,
        name,
        &[Permission::VISIBLE, Permission::YANK_VERSION],
    )
    .await?;

    crate_.yank_version(db, version, true).await?;

    Ok(Json(Response { ok: true }))
}

pub async fn handle_unyank(
    extract::Path((_session_key, name, version)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_ = get_crate_with_permissions(
        db.clone(),
        user,
        name,
        &[Permission::VISIBLE, Permission::YANK_VERSION],
    )
    .await?;

    crate_.yank_version(db, version, false).await?;

    Ok(Json(Response { ok: true }))
}
