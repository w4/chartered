use axum::extract;
use chartered_fs::FileSystem;
use std::str::FromStr;

pub async fn handle(
    extract::Path((_api_key, name, version)): extract::Path<(String, String, String)>,
) -> Vec<u8> {
    let version = chartered_db::get_specific_crate_version(chartered_db::init(), name, version)
        .await
        .unwrap();

    let file_ref = chartered_fs::FileReference::from_str(&version.filesystem_object).unwrap();

    chartered_fs::Local.read(file_ref).await.unwrap()
}
