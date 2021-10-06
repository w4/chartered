use axum::{extract, Json};
use chartered_db::{users::User, ConnectionPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize)]
pub struct RequestParams {
    q: String,
}

#[derive(Serialize)]
pub struct Response {
    users: Vec<ResponseUser>,
}

#[derive(Serialize)]
pub struct ResponseUser {
    user_uuid: chartered_db::uuid::Uuid,
    username: String,
}

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Query(req): extract::Query<RequestParams>,
) -> Result<Json<Response>, Error> {
    let users = User::search(db, req.q, 5)
        .await?
        .into_iter()
        .map(|user| ResponseUser {
            user_uuid: user.uuid.0,
            username: user.username,
        })
        .collect();

    Ok(Json(Response { users }))
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
