use axum::http::StatusCode;
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrateFetchError {
    NotFound,
    MissingPermission(chartered_db::users::UserCratePermissionValue),
    Database(#[from] chartered_db::Error),
}

impl CrateFetchError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound | Self::MissingPermission(Permission::VISIBLE) => StatusCode::NOT_FOUND,
            Self::MissingPermission(_) => StatusCode::FORBIDDEN,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for CrateFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotFound | Self::MissingPermission(Permission::VISIBLE) => {
                write!(f, "The requested crate does not exist")
            }
            Self::MissingPermission(v) => {
                write!(f, "You don't have the {:?} permission for this crate", v)
            }
            Self::Database(_) => write!(f, "An error occurred while fetching the crate."),
        }
    }
}

pub async fn get_crate_with_permissions(
    db: ConnectionPool,
    user: Arc<User>,
    crate_name: String,
    required_permissions: &[chartered_db::users::UserCratePermissionValue],
) -> Result<Arc<Crate>, CrateFetchError> {
    let crate_ = Crate::find_by_name(db.clone(), crate_name)
        .await?
        .ok_or(CrateFetchError::NotFound)
        .map(std::sync::Arc::new)?;
    let has_permissions = user.get_crate_permissions(db, crate_.id).await?;

    for required_permission in required_permissions {
        if !has_permissions.contains(*required_permission) {
            return Err(CrateFetchError::MissingPermission(*required_permission));
        }
    }

    Ok(crate_)
}
