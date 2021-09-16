use axum::{extract, Json};
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use chartered_types::cargo::{CrateVersion, CrateVersionMetadata};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to fetch crate file")]
    File(#[from] std::io::Error),
    #[error("The requested crate does not exist")]
    NoCrate,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) | Self::File(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoCrate => StatusCode::NOT_FOUND,
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Path((_api_key, name)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let crate_ = Crate::find_by_name(db.clone(), name)
        .await?
        .ok_or(Error::NoCrate)
        .map(std::sync::Arc::new)?;
    ensure_has_crate_perm!(db, user, crate_, Permission::VISIBLE | -> Error::NoCrate);

    let versions = crate_.clone().versions(db).await?;

    Ok(Json(Response {
        versions: versions
            .into_iter()
            .map(|v| {
                let (inner, meta) = v.into_cargo_format(&crate_);
                ResponseVersion {
                    inner: inner.into_owned(),
                    meta,
                }
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct ResponseVersion {
    #[serde(flatten)]
    meta: CrateVersionMetadata,
    #[serde(flatten)]
    inner: CrateVersion<'static>,
}

#[derive(Serialize)]
pub struct Response {
    versions: Vec<ResponseVersion>,
}
