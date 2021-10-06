use axum::{extract, Json};
use chartered_db::{users::User, ConnectionPool};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
pub struct Response {
    uuid: chartered_db::uuid::Uuid,
    username: String,
    name: Option<String>,
    nick: Option<String>,
    email: Option<String>,
    external_profile_url: Option<String>,
    picture_url: Option<String>,
}

impl From<chartered_db::users::User> for Response {
    fn from(user: chartered_db::users::User) -> Self {
        Self {
            uuid: user.uuid.0,
            username: user.username,
            name: user.name,
            nick: user.nick,
            email: user.email,
            external_profile_url: user.external_profile_url,
            picture_url: user.picture_url,
        }
    }
}

pub async fn handle(
    extract::Path((_session_key, uuid)): extract::Path<(String, chartered_db::uuid::Uuid)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
) -> Result<Json<Response>, Error> {
    let user = User::find_by_uuid(db, uuid).await?.ok_or(Error::NotFound)?;

    Ok(Json(user.into()))
}

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
