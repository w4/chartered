//! Called by `cargo owners` to get a list of owners of a particular crate. In our case, however,
//! an 'owner' is quite ambiguous as a _person_ isn't directly responsible for a crate, an
//! _organisation_ is. But for the sake of returning some sort of valuable data we'll just return
//! anyone with the `MANAGE_USERS` permission.

use axum::{extract, Json};
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle_get(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    // grab all users with the `MANAGE_USERS` permission for the crate
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
