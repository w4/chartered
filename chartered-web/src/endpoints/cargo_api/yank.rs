use axum::{extract, Json};
use chartered_db::{
    crates::Crate,
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
    #[error("The requested crate does not exist")]
    NoCrate,
    #[error("You don't have {0:?} permission for this crate")]
    NoPermission(Permission),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoCrate => StatusCode::NOT_FOUND,
            Self::NoPermission(_) => StatusCode::FORBIDDEN,
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
    let crate_ = Crate::find_by_name(db.clone(), name)
        .await?
        .ok_or(Error::NoCrate)
        .map(std::sync::Arc::new)?;
    ensure_has_crate_perm!(
        db, user, crate_,
        Permission::VISIBLE | -> Error::NoCrate,
        Permission::YANK_VERSION | -> Error::NoPermission(Permission::YANK_VERSION),
    );

    crate_.yank_version(db, version, true).await?;

    Ok(Json(Response { ok: true }))
}

pub async fn handle_unyank(
    extract::Path((_session_key, name, version)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_ = Crate::find_by_name(db.clone(), name)
        .await?
        .ok_or(Error::NoCrate)
        .map(std::sync::Arc::new)?;
    ensure_has_crate_perm!(
        db, user, crate_,
        Permission::VISIBLE | -> Error::NoCrate,
        Permission::YANK_VERSION | -> Error::NoPermission(Permission::YANK_VERSION),
    );

    crate_.yank_version(db, version, false).await?;

    Ok(Json(Response { ok: true }))
}
