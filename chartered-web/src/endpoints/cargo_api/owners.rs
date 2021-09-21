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
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let users = crate_with_permissions
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
