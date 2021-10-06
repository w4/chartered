mod info;
mod search;

use axum::{
    body::{Body, BoxBody},
    handler::get,
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
        .route("/search", get(search::handle))
        .route("/info/:uuid", get(info::handle)))
}
