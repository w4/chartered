use axum::{body::Full, extract, response::IntoResponse, Json};
use bytes::Bytes;
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use chartered_types::cargo::CrateVersion;
use chrono::TimeZone;
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

pub async fn handle(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<axum::http::Response<Full<Bytes>>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let versions = crate_with_permissions
        .clone()
        .versions_with_uploader(db)
        .await?;

    // returning a Response instead of Json here so we don't have to close
    // every Crate/CrateVersion etc, would be easier if we just had an owned
    // version of each but we're using `spawn_blocking` in chartered-db for
    // diesel which requires `'static' which basically forces us to use Arc
    // if we want to keep a reference to anything ourselves.
    Ok(Json(Response {
        info: (&crate_with_permissions.crate_).into(),
        versions: versions
            .into_iter()
            .map(|(v, user)| ResponseVersion {
                size: v.size,
                created_at: chrono::Utc.from_local_datetime(&v.created_at).unwrap(),
                inner: v.into_cargo_format(&crate_with_permissions.crate_),
                uploader: user.username,
            })
            .collect(),
    })
    .into_response())
}

#[derive(Serialize)]
pub struct Response<'a> {
    #[serde(flatten)]
    info: ResponseInfo<'a>,
    versions: Vec<ResponseVersion<'a>>,
}

#[derive(Serialize)]
pub struct ResponseVersion<'a> {
    #[serde(flatten)]
    inner: CrateVersion<'a>,
    size: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    uploader: String,
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
