mod auth;
mod crates;
mod organisations;
mod ssh_key;
mod users;

use axum::{
    body::{Body, BoxBody},
    handler::{delete, get, put},
    http::{Request, Response},
    Router,
};
use futures::future::Future;
use std::convert::Infallible;

pub fn authenticated_routes() -> Router<
    impl tower::Service<
            Request<Body>,
            Response = Response<BoxBody>,
            Error = Infallible,
            Future = impl Future<Output = Result<Response<BoxBody>, Infallible>> + Send,
        > + Clone
        + Send,
> {
    crate::axum_box_after_every_route!(Router::new()
        .nest("/organisations", organisations::routes())
        .nest("/crates", crates::routes())
        .nest("/users", users::routes())
        .route("/ssh-key", get(ssh_key::handle_get))
        .route("/ssh-key", put(ssh_key::handle_put))
        .route("/ssh-key/:id", delete(ssh_key::handle_delete)))
}

pub fn unauthenticated_routes() -> Router<
    impl tower::Service<
            Request<Body>,
            Response = Response<BoxBody>,
            Error = Infallible,
            Future = impl Future<Output = Result<Response<BoxBody>, Infallible>> + Send,
        > + Clone
        + Send,
> {
    crate::axum_box_after_every_route!(Router::new().nest("/login", auth::routes()))
}
