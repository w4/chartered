use axum::{
    body::{Body, BoxBody},
    extract,
    handler::{get, post},
    http::{Request, Response},
    Router,
};
use chartered_db::{
    users::{User, UserSession},
    ConnectionPool,
};
use futures::future::Future;
use serde::Serialize;
use std::convert::Infallible;

pub mod openid;
pub mod password;

pub fn routes() -> Router<
    impl tower::Service<
            Request<Body>,
            Response = Response<BoxBody>,
            Error = Infallible,
            Future = impl Future<Output = Result<Response<BoxBody>, Infallible>> + Send,
        > + Clone
        + Send,
> {
    crate::axum_box_after_every_route!(Router::new()
        .route("/password", post(password::handle))
        .route("/oauth/:provider/begin", get(openid::begin_oidc))
        .route("/oauth/complete", get(openid::complete_oidc))
        .route("/oauth/providers", get(openid::list_providers)))
}

#[derive(Serialize)]
pub struct LoginResponse {
    key: String,
    expires: chrono::DateTime<chrono::Utc>,
}

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
        key: key.session_key,
        expires,
    })
}
