use axum::{extract, Json};
use chartered_db::{
    users::{User, UserSession},
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
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    extract::ConnectInfo(addr): extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<Response>, Error> {
    // TODO: passwords
    let user = User::find_by_username(db.clone(), req.username)
        .await?
        .ok_or(Error::UnknownUser)?;

    let user_agent = if let Some(extract::TypedHeader(user_agent)) = user_agent {
        Some(user_agent.as_str().to_string())
    } else {
        None
    };

    // todo: session? ip storage? etc...
    let expires = chrono::Utc::now() + chrono::Duration::hours(1);
    let key = UserSession::generate(
        db,
        user.id,
        None,
        Some(expires.naive_utc()),
        user_agent,
        Some(addr.to_string()),
    )
    .await?;

    Ok(Json(Response {
        key: key.session_key,
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
