mod crud;
mod info;
mod list;
mod members;

use crate::middleware::rate_limit::RateLimit;
use axum::{
    handler::Handler,
    routing::{get, patch},
    Router,
};

pub fn routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/",
            get(list::handle_get.layer(rate_limit.with_cost(1)))
                .put(crud::handle_put.layer(rate_limit.with_cost(100))),
        )
        .route(
            "/:org",
            get(info::handle_get.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/:org/members",
            patch(members::handle_patch)
                .put(members::handle_put)
                .delete(members::handle_delete)
                .layer(rate_limit.with_cost(10)),
        )
}
