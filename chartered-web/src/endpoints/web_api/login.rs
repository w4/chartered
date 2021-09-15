use axum::{extract, Json};
use chartered_db::{
    users::{User, UserApiKey},
    ConnectionPool,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("Invalid username/password")]
    UnknownUser,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownUser => StatusCode::FORBIDDEN,
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Json(req): extract::Json<Request>,
) -> Result<Json<Response>, Error> {
    // TODO: passwords
    let user = User::find_by_username(db.clone(), req.username)
        .await?
        .ok_or(Error::UnknownUser)?;

    // todo: session? ip storage? etc...
    let expires = chrono::Utc::now() + chrono::Duration::hours(1);
    let key = UserApiKey::generate(db, user.id, None, Some(expires.naive_utc())).await?;

    Ok(Json(Response {
        key: key.api_key,
        expires,
    }))
}

#[derive(Deserialize)]
pub struct Request {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct Response {
    key: String,
    expires: chrono::DateTime<chrono::Utc>,
}
