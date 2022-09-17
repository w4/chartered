//! Grabs info about a specific organisation including name, description, and a list of members
//! and a list of permissions that are allowed to be applied to others by the requesting user and
//! also all the crates that belong to the organisation.

use axum::{extract, Json};
use chartered_db::{
    organisations::Organisation, permissions::UserPermission, users::User, ConnectionPool,
};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle_get(
    extract::Path(organisation): extract::Path<String>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<Response>, Error> {
    let organisation =
        Arc::new(Organisation::find_by_name(db.clone(), user.id, organisation).await?);

    // checks if the requesting user has the `MANGE_USERS` permission for this
    // organisation
    let can_manage_users = organisation
        .permissions()
        .contains(UserPermission::MANAGE_USERS);

    // fetch both crates and members for the organisation at the same time
    let (crates, users) = tokio::try_join!(
        organisation.clone().crates(db.clone()),
        organisation.clone().members(db),
    )?;

    Ok(Json(Response {
        description: organisation.organisation().description.to_string(),
        // all the permissions the requesting user can give out for this organisation
        possible_permissions: can_manage_users.then(UserPermission::all),
        implied_permissions: can_manage_users.then(UserPermission::implications),
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
                uuid: user.uuid.0,
                display_name: user.display_name().to_string(),
                picture_url: user.picture_url,
                permissions: can_manage_users.then(|| perms),
            })
            .collect(),
        public: organisation.organisation().public,
    }))
}

#[derive(Serialize)]
pub struct Response {
    description: String,
    possible_permissions: Option<UserPermission>,
    implied_permissions: Option<&'static [[UserPermission; 2]]>,
    crates: Vec<ResponseCrate>,
    members: Vec<ResponseUser>,
    public: bool,
}

#[derive(Serialize)]
pub struct ResponseCrate {
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseUser {
    uuid: chartered_db::uuid::Uuid,
    display_name: String,
    picture_url: Option<String>,
    permissions: Option<UserPermission>,
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
