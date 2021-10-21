//! This module is used for anything called directly by cargo. The base URL for all the routes
//! listed in this module is `/a/:key/o/:organisation/api/v1`.
//!
//! Generally only endpoints listed in the [Web API section of the Cargo book][cargo-book] should
//! be implemented here.
//!
//! [cargo-book]: https://doc.rust-lang.org/cargo/reference/registries.html#web-api

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

// requests are already authenticated before this router
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
