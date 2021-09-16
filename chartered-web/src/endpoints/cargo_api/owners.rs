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
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoCrate => StatusCode::NOT_FOUND,
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
    id: i32,
    login: String,
    name: Option<String>,
}

pub async fn handle_get(
    extract::Path((_session_key, name)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_ = Crate::find_by_name(db.clone(), name)
        .await?
        .ok_or(Error::NoCrate)
        .map(std::sync::Arc::new)?;
    ensure_has_crate_perm!(
        db, user, crate_,
        Permission::VISIBLE | -> Error::NoCrate
    );

    let users = crate_
        .owners(db)
        .await?
        .into_iter()
        .map(|user| GetResponseUser {
            id: user.id,
            login: user.username,
            name: None,
        })
        .collect();

    Ok(Json(GetResponse { users }))
}
