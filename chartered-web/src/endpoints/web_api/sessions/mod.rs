mod delete;
mod list;

use crate::RateLimit;
use axum::{handler::Handler, routing::get, Router};

pub fn routes(rate_limit: &RateLimit) -> Router {
    Router::new().route(
        "/",
        get(list::handle_get.layer(rate_limit.with_cost(1)))
            .delete(delete::handle_delete.layer(rate_limit.with_cost(5))),
    )
}
