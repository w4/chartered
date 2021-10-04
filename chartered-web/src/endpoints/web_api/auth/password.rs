use axum::{extract, Json};
use chartered_db::{users::User, ConnectionPool};
use serde::Deserialize;
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
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    addr: extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<super::LoginResponse>, Error> {
    // TODO: passwords
    let user = User::find_by_username(db.clone(), req.username)
        .await?
        .ok_or(Error::UnknownUser)?;

    Ok(Json(super::login(db, user, user_agent, addr).await?))
}

#[derive(Deserialize)]
pub struct Request {
    username: String,
    password: String,
}
