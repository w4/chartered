mod heatmap;
mod info;
mod search;

use crate::RateLimit;
use axum::{handler::Handler, routing::get, Router};

pub fn routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .route(
            "/search",
            get(search::handle.layer(rate_limit.with_cost(5))),
        )
        .route(
            "/info/:uuid",
            get(info::handle.layer(rate_limit.with_cost(1))),
        )
        .route(
            "/info/:uuid/heatmap",
            get(heatmap::handle.layer(rate_limit.with_cost(1))),
        )
}
