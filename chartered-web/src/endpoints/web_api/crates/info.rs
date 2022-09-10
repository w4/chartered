//! Grabs some info about the crate that can be shown in the web UI, like READMEs, descriptions,
//! versions, etc. We group them all together into a single response as we can fetch them in
//! a single query and the users don't have to make multiple calls out.
//!
//! Unlike crates.io, we're only keeping the _latest_ README pushed to the crate, so there's no
//! need to have version-specific info responses - we'll just send an overview of each one.

use axum::{extract, response::IntoResponse, Json};
use chartered_db::{crates::Crate, permissions::UserPermission, users::User, ConnectionPool};
use chartered_types::cargo::CrateVersion;
use chrono::TimeZone;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

pub async fn handle(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<axum::response::Response, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    // grab all versions of this crate and the person who uploaded them
    let versions = crate_with_permissions
        .clone()
        .versions_with_uploader(db)
        .await?;

    Ok(Json(Response {
        info: (&crate_with_permissions.crate_).into(),
        versions: versions
            .into_iter()
            .map(|(v, user)| ResponseVersion {
                size: v.size,
                created_at: chrono::Utc.from_local_datetime(&v.created_at).unwrap(),
                inner: v.into_cargo_format(&crate_with_permissions.crate_),
                uploader: ResponseVersionUploader {
                    uuid: user.uuid.0,
                    display_name: user.display_name().to_string(),
                    picture_url: user.picture_url,
                },
            })
            .collect(),
        permissions: crate_with_permissions.permissions,
    })
    // returning a Response instead of Json here so we don't have to clone
    // every Crate/CrateVersion etc, would be easier if we just had an owned
    // version of each but we're using `spawn_blocking` in chartered-db for
    // diesel which requires `'static' which basically forces us to use Arc
    // if we want to keep a reference to anything ourselves.
    .into_response())
}

#[derive(Serialize)]
pub struct Response<'a> {
    #[serde(flatten)]
    info: ResponseInfo<'a>,
    versions: Vec<ResponseVersion<'a>>,
    permissions: UserPermission,
}

#[derive(Serialize)]
pub struct ResponseVersion<'a> {
    #[serde(flatten)]
    inner: CrateVersion<'a>,
    size: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    uploader: ResponseVersionUploader,
}

#[derive(Serialize)]
pub struct ResponseVersionUploader {
    uuid: chartered_db::uuid::Uuid,
    display_name: String,
    picture_url: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseInfo<'a> {
    name: &'a str,
    readme: Option<&'a str>,
    description: Option<&'a str>,
    repository: Option<&'a str>,
    homepage: Option<&'a str>,
    documentation: Option<&'a str>,
}

impl<'a> From<&'a Crate> for ResponseInfo<'a> {
    fn from(crate_: &'a Crate) -> Self {
        Self {
            name: &crate_.name,
            readme: crate_.readme.as_deref(),
            description: crate_.description.as_deref(),
            repository: crate_.repository.as_deref(),
            homepage: crate_.homepage.as_deref(),
            documentation: crate_.documentation.as_deref(),
        }
    }
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
