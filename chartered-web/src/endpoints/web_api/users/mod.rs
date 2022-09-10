mod heatmap;
mod info;
mod search;

use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/search", get(search::handle))
        .route("/info/:uuid", get(info::handle))
        .route("/info/:uuid/heatmap", get(heatmap::handle))
}
