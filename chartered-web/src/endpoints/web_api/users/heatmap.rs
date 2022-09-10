use axum::{extract, Json};
use chartered_db::users::User;
use chartered_db::ConnectionPool;
use chrono::NaiveDate;
use std::collections::HashMap;
use thiserror::Error;

pub async fn handle(
    extract::Path((_session_key, uuid)): extract::Path<(String, chartered_db::uuid::Uuid)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
) -> Result<Json<Response>, Error> {
    let user = User::find_by_uuid(db.clone(), uuid)
        .await?
        .ok_or(Error::NotFound)?;

    let res = user
        .published_versions_by_date(db)
        .await?
        .into_iter()
        .map(|(k, v)| (k.date(), v))
        .collect();

    Ok(Json(res))
}

pub type Response = HashMap<NaiveDate, i64>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("User doesn't exist")]
    NotFound,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

define_error_response!(Error);
