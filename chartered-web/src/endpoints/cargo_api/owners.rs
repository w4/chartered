use axum::{extract, Json};
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use serde::Serialize;
use std::sync::Arc;

define_error!(
    Database(_e: chartered_db::Error) => INTERNAL_SERVER_ERROR / "Failed to query database",
    NoCrate => NOT_FOUND / "The requested crate does not exist",
);

#[derive(Serialize)]
pub struct GetResponse {
    users: Vec<GetResponseUser>,
}

#[derive(Serialize)]
pub struct GetResponseUser {
    id: i32,
    login: String,
    name: Option<String>,
}

pub async fn handle_get(
    extract::Path((_api_key, name)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_ = get_crate!(db, name; || -> Error::NoCrate);
    ensure_has_crate_perm!(db, user, crate_, Permission::VISIBLE; || -> Error::NoCrate);

    let users = crate_
        .owners(db)
        .await?
        .into_iter()
        .map(|user| GetResponseUser {
            id: user.id,
            login: user.username,
            name: None,
        })
        .collect();

    Ok(Json(GetResponse { users }))
}
