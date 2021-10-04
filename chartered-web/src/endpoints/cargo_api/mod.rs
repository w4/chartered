mod download;
mod owners;
mod publish;
mod yank;

use axum::{
    body::{Body, BoxBody},
    handler::{delete, get, put},
    http::{Request, Response},
    Router,
};
use futures::future::Future;
use std::convert::Infallible;

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
        .route("/crates/new", put(publish::handle))
        // .route("/crates/search", get(hello_world))
        .route("/crates/:crate/owners", get(owners::handle_get))
        // .route("/crates/:crate/owners", put(hello_world))
        // .route("/crates/:crate/owners", delete(hello_world))
        .route("/crates/:crate/:version/yank", delete(yank::handle_yank))
        .route("/crates/:crate/:version/unyank", put(yank::handle_unyank))
        .route("/crates/:crate/:version/download", get(download::handle)))
}
