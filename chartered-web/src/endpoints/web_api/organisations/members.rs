//! CRUD methods to manage members of an organisation, given the requesting user has the
//! `MANAGE_USERS` permission at the organisation level.

use axum::{extract, Json};
use chartered_db::{
    organisations::Organisation, permissions::UserPermission, users::User, ConnectionPool,
};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

use crate::endpoints::ErrorResponse;

/// Updates an organisation member's permissions
pub async fn handle_patch(
    extract::Path(organisation): extract::Path<String>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let organisation =
        Arc::new(Organisation::find_by_name(db.clone(), user.id, organisation).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    let affected_rows = organisation
        .update_permissions(db, action_user.id, req.permissions)
        .await?;
    if affected_rows == 0 {
        return Err(Error::UpdateConflictRemoved);
    }

    Ok(Json(ErrorResponse { error: None }))
}

/// Adds a new member to the organisation with a given set of permissions.
pub async fn handle_put(
    extract::Path(organisation): extract::Path<String>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let organisation =
        Arc::new(Organisation::find_by_name(db.clone(), user.id, organisation).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    organisation
        .insert_permissions(db, action_user.id, req.permissions)
        .await?;

    Ok(Json(ErrorResponse { error: None }))
}

/// Deletes a member from the organisation entirely
pub async fn handle_delete(
    extract::Path(organisation): extract::Path<String>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<DeleteRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let organisation =
        Arc::new(Organisation::find_by_name(db.clone(), user.id, organisation).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    organisation.delete_member(db, action_user.id).await?;

    Ok(Json(ErrorResponse { error: None }))
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
    #[error("Permissions update conflict, user was removed as a member of the organisation")]
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
