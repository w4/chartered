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

use crate::RateLimit;
use axum::{
    handler::Handler,
    routing::{delete, get, put},
    Router,
};

// requests are already authenticated before this router
pub fn routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/crates/new",
            put(publish::handle.layer(rate_limit.with_cost(200))),
        )
        // .route("/crates/search", get(hello_world))
        .route(
            "/crates/:crate/owners",
            get(owners::handle_get.layer(rate_limit.with_cost(1))),
        )
        // .route("/crates/:crate/owners", put(hello_world))
        // .route("/crates/:crate/owners", delete(hello_world))
        .route(
            "/crates/:crate/:version/yank",
            delete(yank::handle_yank.layer(rate_limit.with_cost(50))),
        )
        .route(
            "/crates/:crate/:version/unyank",
            put(yank::handle_unyank.layer(rate_limit.with_cost(50))),
        )
        .route(
            "/crates/:crate/:version/download",
            get(download::handle.layer(rate_limit.with_cost(1))),
        )
}
