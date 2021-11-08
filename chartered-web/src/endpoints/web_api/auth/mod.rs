use axum::{
    extract,
    routing::{get, post},
    Router,
};
use chartered_db::{
    users::{User, UserSession},
    uuid::Uuid,
    ConnectionPool,
};

use serde::Serialize;

pub mod extend;
pub mod logout;
pub mod openid;
pub mod password;

pub fn authenticated_routes() -> Router {
    Router::new()
        .route("/logout", get(logout::handle))
        .route("/extend", get(extend::handle))
}

pub fn unauthenticated_routes() -> Router {
    Router::new()
        .route("/register/password", post(password::handle_register))
        .route("/login/password", post(password::handle_login))
        .route("/login/oauth/:provider/begin", get(openid::begin_oidc))
        .route("/login/oauth/complete", get(openid::complete_oidc))
        .route("/login/oauth/providers", get(openid::list_providers))
}

#[derive(Serialize)]
pub struct LoginResponse {
    user_uuid: Uuid,
    key: String,
    expires: chrono::DateTime<chrono::Utc>,
    picture_url: Option<String>,
}

/// Takes the given `User` and generates a session for it and returns a response containing an API
/// key to the frontend that it can save for further request
pub async fn login(
    db: ConnectionPool,
    user: User,
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    extract::ConnectInfo(addr): extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<LoginResponse, chartered_db::Error> {
    let user_agent = if let Some(extract::TypedHeader(user_agent)) = user_agent {
        Some(user_agent.as_str().to_string())
    } else {
        None
    };

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

    Ok(LoginResponse {
        user_uuid: user.uuid.0,
        key: key.session_key,
        expires,
        picture_url: user.picture_url,
    })
}
