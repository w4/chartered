mod auth;
mod crates;
mod organisations;
mod ssh_key;
mod users;

use axum::{
    routing::{delete, get},
    Router,
};

pub fn authenticated_routes() -> Router {
    Router::new()
        .nest("/organisations", organisations::routes())
        .nest("/crates", crates::routes())
        .nest("/users", users::routes())
        .nest("/auth", auth::authenticated_routes())
        .route(
            "/ssh-key",
            get(ssh_key::handle_get).put(ssh_key::handle_put),
        )
        .route("/ssh-key/:id", delete(ssh_key::handle_delete))
}

pub fn unauthenticated_routes() -> Router {
    Router::new().nest("/auth", auth::unauthenticated_routes())
}
