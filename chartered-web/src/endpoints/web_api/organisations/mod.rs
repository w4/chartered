mod crud;
mod info;
mod list;
mod members;

use axum::{
    routing::{get, patch},
    Router,
};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list::handle_get).put(crud::handle_put))
        .route("/:org", get(info::handle_get))
        .route(
            "/:org/members",
            patch(members::handle_patch)
                .put(members::handle_put)
                .delete(members::handle_delete),
        )
}
