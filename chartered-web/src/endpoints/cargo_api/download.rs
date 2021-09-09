use axum::extract;
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use chartered_fs::FileSystem;
use std::{str::FromStr, sync::Arc};

define_error!(
    Database(_e: chartered_db::Error) => INTERNAL_SERVER_ERROR / "Failed to query database",
    File(_e: std::io::Error) => INTERNAL_SERVER_ERROR / "Failed to fetch crate file",
    NoVersion => NOT_FOUND / "The requested version does not exist for the crate",
    NoCrate => NOT_FOUND / "The requested crate does not exist",
);

pub async fn handle(
    extract::Path((_api_key, name, version)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Vec<u8>, Error> {
    let crate_ = get_crate!(db, name; || -> Error::NoCrate);
    ensure_has_crate_perm!(db, user, crate_, Permission::VISIBLE; || -> Error::NoCrate);

    let version = crate_.version(db, version).await?.ok_or(Error::NoVersion)?;

    let file_ref = chartered_fs::FileReference::from_str(&version.filesystem_object).unwrap();

    Ok(chartered_fs::Local.read(file_ref).await?)
}
