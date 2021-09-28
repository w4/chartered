use axum::{extract, Json};
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

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

#[derive(Serialize)]
pub struct Response {
    ok: bool,
}

pub async fn handle_yank(
    extract::Path((_session_key, organisation, name, version)): extract::Path<(
        String,
        String,
        String,
        String,
    )>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    crate_with_permissions
        .yank_version(db, version, true)
        .await?;

    Ok(Json(Response { ok: true }))
}

pub async fn handle_unyank(
    extract::Path((_session_key, organisation, name, version)): extract::Path<(
        String,
        String,
        String,
        String,
    )>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    crate_with_permissions
        .yank_version(db, version, false)
        .await?;

    Ok(Json(Response { ok: true }))
}
