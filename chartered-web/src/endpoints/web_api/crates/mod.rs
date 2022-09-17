mod info;
mod members;
mod most_downloaded;
mod recently_created;
mod recently_updated;
mod search;

use crate::middleware::rate_limit::RateLimit;
use axum::handler::Handler;
use axum::{routing::get, Router};

pub fn routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/:org/:crate",
            get(info::handle.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/:org/:crate/members",
            get(members::handle_get.layer(rate_limit.with_cost(1)))
                .patch(members::handle_patch.layer(rate_limit.with_cost(10)))
                .put(members::handle_put.layer(rate_limit.with_cost(10)))
                .delete(members::handle_delete.layer(rate_limit.with_cost(10))),
        )
        .route(
            "/recently-updated",
            get(recently_updated::handle.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/recently-created",
            get(recently_created::handle.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/most-downloaded",
            get(most_downloaded::handle.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/search",
            get(search::handle.layer(rate_limit.with_cost(5))),
        )
}
