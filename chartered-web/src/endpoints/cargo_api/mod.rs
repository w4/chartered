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
    routing::{delete, get, put},
    Router,
};

// requests are already authenticated before this router
pub fn routes() -> Router {
    Router::new()
        .route("/crates/new", put(publish::handle))
        // .route("/crates/search", get(hello_world))
        .route("/crates/:crate/owners", get(owners::handle_get))
        // .route("/crates/:crate/owners", put(hello_world))
        // .route("/crates/:crate/owners", delete(hello_world))
        .route("/crates/:crate/:version/yank", delete(yank::handle_yank))
        .route("/crates/:crate/:version/unyank", put(yank::handle_unyank))
        .route("/crates/:crate/:version/download", get(download::handle))
}
