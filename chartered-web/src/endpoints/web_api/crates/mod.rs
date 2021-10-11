mod info;
mod members;
mod most_downloaded;
mod recently_updated;
mod search;
mod recently_created;

use axum::{
    body::{Body, BoxBody},
    handler::{delete, get, patch, put},
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
        .route("/:org/:crate", get(info::handle))
        .route("/:org/:crate/members", get(members::handle_get))
        .route("/:org/:crate/members", patch(members::handle_patch))
        .route("/:org/:crate/members", put(members::handle_put))
        .route("/:org/:crate/members", delete(members::handle_delete))
        .route("/recently-updated", get(recently_updated::handle))
        .route("/recently-created", get(recently_created::handle))
        .route("/most-downloaded", get(most_downloaded::handle))
        .route("/search", get(search::handle)))
}
