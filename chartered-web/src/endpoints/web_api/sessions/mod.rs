mod list;

use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new().route("/", get(list::handle_get))
}
