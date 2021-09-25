use axum::{extract, Json};
use chartered_db::{
    organisations::Organisation, permissions::UserPermission, users::User, ConnectionPool,
};
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

pub async fn handle_get(
    extract::Path((_session_key, organisation)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let organisation =
        Arc::new(Organisation::find_by_name(db.clone(), user.id, organisation).await?);

    let can_manage_users = organisation
        .permissions()
        .contains(UserPermission::MANAGE_USERS);

    let (crates, users) = tokio::try_join!(
        organisation.clone().crates(db.clone()),
        organisation.clone().members(db),
    )?;

    Ok(Json(Response {
        description: organisation.organisation().description.to_string(),
        possible_permissions: can_manage_users.then(UserPermission::all),
        crates: crates
            .into_iter()
            .map(|v| ResponseCrate {
                name: v.name,
                description: v.description,
            })
            .collect(),
        members: users
            .into_iter()
            .map(|(user, perms)| ResponseUser {
                uuid: user.uuid.to_string(),
                username: user.username,
                permissions: can_manage_users.then(|| perms),
            })
            .collect(),
    }))
}

#[derive(Serialize)]
pub struct Response {
    description: String,
    possible_permissions: Option<UserPermission>,
    crates: Vec<ResponseCrate>,
    members: Vec<ResponseUser>,
}

#[derive(Serialize)]
pub struct ResponseCrate {
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseUser {
    uuid: String,
    username: String,
    permissions: Option<UserPermission>,
}
