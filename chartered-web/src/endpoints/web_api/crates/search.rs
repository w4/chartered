use axum::{extract, Json};
use chartered_db::{crates::Crate, permissions::UserPermission, users::User, ConnectionPool};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Deserialize)]
pub struct RequestParams {
    q: String,
}

#[derive(Serialize)]
pub struct Response {
    crates: Vec<ResponseCrate>,
}

#[derive(Serialize)]
pub struct ResponseCrate {
    organisation: String,
    name: String,
    description: Option<String>,
    version: String,
    homepage: Option<String>,
    repository: Option<String>,
    updated: DateTime<Utc>,
    permissions: UserPermission,
}

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Query(req): extract::Query<RequestParams>,
) -> Result<Json<Response>, Error> {
    let crates = Crate::search(db, user.id, req.q, 5)
        .await?
        .into_iter()
        .map(|(org, crates_with_permissions)| {
            crates_with_permissions
                .into_iter()
                .map(move |v| ResponseCrate {
                    organisation: org.name.to_string(),
                    name: v.crate_.name,
                    description: v.crate_.description,
                    version: "test".to_string(),
                    homepage: v.crate_.homepage,
                    repository: v.crate_.repository,
                    updated: Utc::now(), // todo
                    permissions: v.permissions,
                })
        })
        .flatten()
        .collect();

    Ok(Json(Response { crates }))
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

define_error_response!(Error);
