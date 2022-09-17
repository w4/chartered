mod auth;
mod crates;
mod organisations;
mod sessions;
mod ssh_key;
mod users;

use crate::RateLimit;
use axum::{
    handler::Handler,
    routing::{delete, get},
    Router,
};

pub fn authenticated_routes(rate_limit: &RateLimit) -> Router {
    Router::new()
        .nest("/organisations", organisations::routes(rate_limit))
        .nest("/crates", crates::routes(rate_limit))
        .nest("/users", users::routes(rate_limit))
        .nest("/auth", auth::authenticated_routes(rate_limit))
        .nest("/sessions", sessions::routes(rate_limit))
        .route(
            "/ssh-key",
            get(ssh_key::handle_get.layer(rate_limit.with_cost(1)))
                .put(ssh_key::handle_put.layer(rate_limit.with_cost(24))),
        )
        .route(
            "/ssh-key/:id",
            delete(ssh_key::handle_delete.layer(rate_limit.with_cost(24))),
        )
}

pub fn unauthenticated_routes(rate_limit: &RateLimit) -> Router {
    Router::new().nest("/auth", auth::unauthenticated_routes(rate_limit))
}
