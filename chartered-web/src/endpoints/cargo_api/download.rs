use axum::extract;
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use chartered_fs::FileSystem;
use std::{str::FromStr, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to fetch crate file")]
    File(#[from] std::io::Error),
    #[error("The requested version does not exist for the crate")]
    NoVersion,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::File(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoVersion => StatusCode::NOT_FOUND,
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Path((_session_key, organisation, name, version)): extract::Path<(
        String,
        String,
        String,
        String,
    )>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Vec<u8>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let version = crate_with_permissions
        .version(db, version)
        .await?
        .ok_or(Error::NoVersion)?;

    let file_ref = chartered_fs::FileReference::from_str(&version.filesystem_object).unwrap();

    Ok(chartered_fs::Local.read(file_ref).await?)
}
