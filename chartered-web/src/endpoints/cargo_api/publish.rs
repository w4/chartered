use axum::extract;
use bytes::Bytes;
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use chartered_fs::FileSystem;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{convert::TryInto, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("The requested crate does not exist")]
    NoCrate,
    #[error("You don't have permission to publish versions for this crate")]
    NoPermission,
    #[error("Invalid JSON from client: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Invalid body")]
    MetadataParse,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoCrate => StatusCode::NOT_FOUND,
            Self::NoPermission => StatusCode::FORBIDDEN,
            Self::JsonParse(_) | Self::MetadataParse => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);

#[derive(Serialize, Debug, Default)]
pub struct PublishCrateResponse {
    warnings: PublishCrateResponseWarnings,
}

#[derive(Serialize, Debug, Default)]
pub struct PublishCrateResponseWarnings {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    body: Bytes,
) -> Result<axum::response::Json<PublishCrateResponse>, Error> {
    let (_, (metadata_bytes, crate_bytes)) =
        parse(body.as_ref()).map_err(|_| Error::MetadataParse)?;
    let metadata: Metadata = serde_json::from_slice(metadata_bytes)?;

    let crate_ = Crate::find_by_name(db.clone(), metadata.inner.name.to_string())
        .await?
        .ok_or(Error::NoCrate)
        .map(std::sync::Arc::new)?;
    ensure_has_crate_perm!(
        db, user, crate_,
        Permission::VISIBLE | -> Error::NoCrate,
        Permission::PUBLISH_VERSION | -> Error::NoPermission,
    );

    let file_ref = chartered_fs::Local.write(crate_bytes).await.unwrap();

    crate_
        .publish_version(
            db,
            file_ref,
            hex::encode(Sha256::digest(crate_bytes)),
            metadata.inner.into_owned(),
        )
        .await?;

    Ok(axum::response::Json(PublishCrateResponse::default()))
}

fn parse(body: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8])> {
    use nom::{bytes::complete::take, combinator::map_res};
    use std::array::TryFromSliceError;

    let u32_from_le_bytes =
        |b: &[u8]| Ok::<_, TryFromSliceError>(u32::from_le_bytes(b.try_into()?));
    let mut read_u32 = map_res(take(4_usize), u32_from_le_bytes);

    let (rest, metadata_length) = read_u32(body)?;
    let (rest, metadata_bytes) = take(metadata_length)(rest)?;
    let (rest, crate_length) = read_u32(rest)?;
    let (rest, crate_bytes) = take(crate_length)(rest)?;

    Ok((rest, (metadata_bytes, crate_bytes)))
}

#[derive(Deserialize, Debug)]
pub struct Metadata<'a> {
    authors: Vec<&'a str>,
    description: Option<&'a str>,
    documentation: Option<&'a str>,
    homepage: Option<&'a str>,
    readme: Option<&'a str>,
    readme_file: Option<&'a str>,
    keywords: Vec<&'a str>,
    categories: Vec<&'a str>,
    license: Option<&'a str>,
    license_file: Option<&'a str>,
    repository: Option<&'a str>,
    #[serde(flatten)]
    inner: chartered_types::cargo::CrateVersion<'a>,
}
