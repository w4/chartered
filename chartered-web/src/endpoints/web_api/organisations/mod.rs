mod crud;
mod info;
mod list;
mod members;

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
        .route("/", get(list::handle_get))
        .route("/", put(crud::handle_put))
        .route("/:org", get(info::handle_get))
        .route("/:org/members", patch(members::handle_patch))
        .route("/:org/members", put(members::handle_put))
        .route("/:org/members", delete(members::handle_delete)))
}
