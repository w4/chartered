//! Handles crate-level member overrides that can add permissions on top of organisations.
//!
//! This is essentially a CRUD controller, nice and easy one.

use axum::{extract, Json};
use chartered_db::{
    crates::Crate, permissions::UserPermission, users::User, uuid::Uuid, ConnectionPool,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::endpoints::ErrorResponse;

/// Lists all crate-level members and the permissions they have.
///
/// These members could be specific to the crate or they could be overrides ontop of the org.
pub async fn handle_get(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let members = crate_with_permissions
        .members(db)
        .await?
        .into_iter()
        .map(|(user, permissions)| GetResponseMember {
            uuid: user.uuid.0,
            display_name: user.display_name().to_string(),
            picture_url: user.picture_url,
            permissions,
        })
        .collect();

    Ok(Json(GetResponse {
        possible_permissions: UserPermission::names(),
        implied_permissions: UserPermission::implications(),
        members,
    }))
}

/// Updates a crate member's permissions
pub async fn handle_patch(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    let affected_rows = crate_with_permissions
        .update_permissions(db, action_user.id, req.permissions)
        .await?;
    if affected_rows == 0 {
        return Err(Error::UpdateConflictRemoved);
    }

    Ok(Json(ErrorResponse { error: None }))
}

/// Inserts an permissions override for this crate for a specific user
pub async fn handle_put(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    crate_with_permissions
        .insert_permissions(db, action_user.id, req.permissions)
        .await?;

    Ok(Json(ErrorResponse { error: None }))
}

/// Deletes a member override from this crate
pub async fn handle_delete(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<DeleteRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    crate_with_permissions
        .delete_member(db, action_user.id)
        .await?;

    Ok(Json(ErrorResponse { error: None }))
}

#[derive(Serialize)]
pub struct GetResponse {
    possible_permissions: &'static [&'static str],
    implied_permissions: &'static [[UserPermission; 2]],
    members: Vec<GetResponseMember>,
}

#[derive(Deserialize, Serialize)]
pub struct GetResponseMember {
    uuid: Uuid,
    display_name: String,
    picture_url: Option<String>,
    permissions: UserPermission,
}

#[derive(Deserialize)]
pub struct PutOrPatchRequest {
    user_uuid: chartered_db::uuid::Uuid,
    permissions: UserPermission,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    user_uuid: chartered_db::uuid::Uuid,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Permissions update conflict, user was removed as a member of the crate")]
    UpdateConflictRemoved,
    #[error("An invalid user id was given")]
    InvalidUserId,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::UpdateConflictRemoved => StatusCode::CONFLICT,
            Self::InvalidUserId => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);
