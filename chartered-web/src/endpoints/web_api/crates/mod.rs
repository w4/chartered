mod info;
mod members;
mod most_downloaded;
mod recently_created;
mod recently_updated;
mod search;

use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/:org/:crate", get(info::handle))
        .route(
            "/:org/:crate/members",
            get(members::handle_get)
                .patch(members::handle_patch)
                .put(members::handle_put)
                .delete(members::handle_delete),
        )
        .route("/recently-updated", get(recently_updated::handle))
        .route("/recently-created", get(recently_created::handle))
        .route("/most-downloaded", get(most_downloaded::handle))
        .route("/search", get(search::handle))
}
