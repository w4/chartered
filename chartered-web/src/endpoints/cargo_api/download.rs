use axum::extract;
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermission, UserCratePermissionValue},
    ConnectionPool,
};
use chartered_fs::FileSystem;
use std::{str::FromStr, sync::Arc};

define_error!(
    Database(_e: chartered_db::Error) => INTERNAL_SERVER_ERROR / "Failed to query database",
    File(_e: std::io::Error) => INTERNAL_SERVER_ERROR / "Failed to fetch crate file",
    NoVersion => NOT_FOUND / "That requested version does not exist for the crate",
    NoCrate => NOT_FOUND / "The requested crate does not exist",
);

pub async fn handle(
    extract::Path((_api_key, name, version)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Vec<u8>, Error> {
    let c = Crate::find_by_name(db.clone(), name)
        .await?
        .ok_or(Error::NoCrate)?;

    let perms = UserCratePermission::find(db.clone(), user.id, c.id)
        .await?
        .unwrap_or_default();

    if !perms
        .permissions
        .contains(UserCratePermissionValue::VISIBLE)
    {
        return Err(Error::NoCrate);
    }

    let version = Arc::new(c)
        .version(db, version)
        .await?
        .ok_or(Error::NoVersion)?;

    let file_ref = chartered_fs::FileReference::from_str(&version.filesystem_object).unwrap();

    Ok(chartered_fs::Local.read(file_ref).await?)
}
