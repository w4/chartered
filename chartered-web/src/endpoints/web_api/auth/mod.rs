use crate::middleware::rate_limit::RateLimit;

use axum::{
    extract,
    handler::Handler,
    routing::{get, post},
    Extension, Router,
};
use chartered_db::{
    users::{User, UserSession},
    uuid::Uuid,
    ConnectionPool,
};
use serde::Serialize;

use std::net::IpAddr;

pub mod extend;
pub mod logout;
pub mod openid;
pub mod password;

pub fn authenticated_routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/logout",
            get(logout::handle.layer(rate_limit.with_cost(5))),
        )
        .route(
            "/extend",
            get(extend::handle.layer(rate_limit.with_cost(1))),
        )
}

pub fn unauthenticated_routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/register/password",
            post(password::handle_register.layer(rate_limit.with_cost(200))),
        )
        .route(
            "/login/password",
            post(password::handle_login.layer(rate_limit.with_cost(100))),
        )
        .route(
            "/login/oauth/:provider/begin",
            get(openid::begin_oidc.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/login/oauth/complete",
            get(openid::complete_oidc.layer(rate_limit.with_cost(75))),
        )
        .route(
            "/login/oauth/providers",
            get(openid::list_providers.layer(rate_limit.with_cost(1))),
        )
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
    Extension(addr): Extension<IpAddr>,
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
