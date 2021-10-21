//! Publishes a crate version
//!
//! This can also potentially create the crate under the given organisation if the crate doesn't
//! already exist and the user has the `CREATE_CRATE` permissions.

use axum::extract;
use bytes::Bytes;
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use chartered_fs::FileSystem;
use chartered_types::cargo::{CrateDependency, CrateFeatures, CrateVersion};
use nom_bytes::BytesWrapper;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, convert::TryInto, sync::Arc};
use thiserror::Error;

pub async fn handle(
    extract::Path((_session_key, organisation)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Extension(fs): extract::Extension<Arc<FileSystem>>,
    body: Bytes,
) -> Result<axum::response::Json<PublishCrateResponse>, Error> {
    // cargo sends the crate metadata and the crate itself packed together, we'll parse these
    // two separate bits of data out
    let (_, (metadata_bytes, crate_bytes)) = parse(body).map_err(|_| Error::MetadataParse)?;
    let metadata: Metadata<'_> = serde_json::from_slice(&metadata_bytes)?;

    // validates the crate has a valid name, crates.io imposes some sane restrictions
    // so we'll just use those
    if !validate_crate_name(&metadata.inner.name) {
        return Err(Error::InvalidCrateName);
    }

    // looks up the crate, though we won't error on it just yet
    let crate_with_permissions = Crate::find_by_name(
        db.clone(),
        user.id,
        organisation.clone(),
        metadata.inner.name.to_string(),
    )
    .await;

    // if we failed to lookup the crate because it was missing, we'll create a new one fresh
    // if we have the permissions, that is. `Crate::create` will check those for us
    let crate_with_permissions = match crate_with_permissions {
        Ok(v) => Arc::new(v),
        Err(chartered_db::Error::MissingCrate) => {
            let new_crate = Crate::create(
                db.clone(),
                user.id,
                organisation,
                metadata.inner.name.to_string(),
            )
            .await?;
            Arc::new(new_crate)
        }
        Err(e) => return Err(e.into()),
    };

    // take a checksum of the crate to write to the database to ensure integrity
    let checksum = hex::encode(Sha256::digest(&crate_bytes));

    // writes the file to the filesystem and takes a `FileReference` we can store in the
    // db to.. reference this file when it's needed (ie. on download)
    let file_ref = fs.write(crate_bytes).await.map_err(Box::new)?;

    // and finally, publish the version!
    crate_with_permissions
        .publish_version(
            db,
            user,
            file_ref,
            checksum,
            metadata_bytes.len().try_into().unwrap(),
            metadata.inner.into(),
            metadata.meta,
        )
        .await?;

    Ok(axum::response::Json(PublishCrateResponse::default()))
}

/// Cargo sends the metadata and crate packed together and prepended with a single `u32`
/// representing how many bytes are in the next section, we'll parse these two byte chunks
/// out and return them `(Metadata, Crate)`.
fn parse(body: impl Into<BytesWrapper>) -> nom::IResult<BytesWrapper, (Bytes, Bytes)> {
    use nom::{bytes::complete::take, combinator::map_res};
    use std::array::TryFromSliceError;

    let body = body.into();

    let u32_from_le_bytes =
        |b: BytesWrapper| Ok::<_, TryFromSliceError>(u32::from_le_bytes((&b[..]).try_into()?));
    let mut read_u32 = map_res(take(4_usize), u32_from_le_bytes);

    let (rest, metadata_length) = read_u32(body)?;
    let (rest, metadata_bytes) = take(metadata_length)(rest)?;
    let (rest, crate_length) = read_u32(rest)?;
    let (rest, crate_bytes) = take(crate_length)(rest)?;

    Ok((rest, (metadata_bytes.into(), crate_bytes.into())))
}

/// Most of these limitations are copied from crates.io as they're rather sane defaults,
/// though not all of them are implemented yet, a non-comprehensive list is available [here][list]
///
/// [list]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
fn validate_crate_name(name: &str) -> bool {
    const MAX_NAME_LENGTH: usize = 64;

    let starts_with_alphabetic = name
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or_default();
    let is_alphanumeric = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    let is_under_max_length = name.len() < MAX_NAME_LENGTH;

    starts_with_alphabetic && is_alphanumeric && is_under_max_length
}

/// Some metadata about the crate, sent to us by the user's `cargo` CLI
#[allow(dead_code)] // a lot of these need checking/validating
#[derive(Deserialize, Debug)]
pub struct Metadata<'a> {
    #[serde(borrow)]
    authors: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    readme_file: Option<Cow<'a, str>>,
    #[serde(borrow)]
    keywords: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    categories: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    license: Option<Cow<'a, str>>,
    #[serde(borrow)]
    license_file: Option<Cow<'a, str>>,
    #[serde(flatten)]
    meta: chartered_types::cargo::CrateVersionMetadata,
    #[serde(flatten)]
    inner: MetadataCrateVersion<'a>,
}

/// Some specific metadata about the crate version currently being pushed to us
#[derive(Deserialize, Debug)]
pub struct MetadataCrateVersion<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub vers: Cow<'a, str>,
    pub deps: Vec<MetadataCrateDependency<'a>>,
    pub features: CrateFeatures,
    #[serde(borrow)]
    pub links: Option<Cow<'a, str>>,
}

impl From<MetadataCrateVersion<'_>> for CrateVersion<'static> {
    fn from(us: MetadataCrateVersion<'_>) -> Self {
        Self {
            name: Cow::Owned(us.name.into_owned()),
            vers: Cow::Owned(us.vers.into_owned()),
            deps: us.deps.into_iter().map(CrateDependency::from).collect(),
            features: us.features,
            links: us.links.map(|v| Cow::Owned(v.into_owned())),
        }
    }
}

/// We've redefined `MetadataCrateDependency` for deserialisation because `cargo publish` passes
/// a `version_req`, whereas when downloading it expects `req` - and `package` isn't returned.
#[derive(Deserialize, Debug)]
pub struct MetadataCrateDependency<'a> {
    pub name: Cow<'a, str>,
    pub version_req: Cow<'a, str>, // needs to be: https://github.com/steveklabnik/semver#requirements
    pub features: Vec<Cow<'a, str>>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<Cow<'a, str>>, // a string such as "cfg(windows)"
    pub kind: Cow<'a, str>,           // dev, build or normal
    pub registry: Option<Cow<'a, str>>,
    pub explicit_name_in_toml: Option<Cow<'a, str>>,
}

impl From<MetadataCrateDependency<'_>> for CrateDependency<'static> {
    fn from(us: MetadataCrateDependency<'_>) -> CrateDependency<'static> {
        #[allow(clippy::option_if_let_else)] // us.name can't be moved into both closures
        let (name, package) = if let Some(explicit_name_in_toml) = us.explicit_name_in_toml {
            (
                explicit_name_in_toml.into_owned(),
                Some(us.name.into_owned()),
            )
        } else {
            (us.name.into_owned(), None)
        };

        Self {
            name: Cow::Owned(name),
            req: Cow::Owned(us.version_req.into_owned()),
            features: us
                .features
                .into_iter()
                .map(|v| Cow::Owned(v.into_owned()))
                .collect(),
            optional: us.optional,
            default_features: us.default_features,
            target: us.target.map(|v| Cow::Owned(v.into_owned())),
            kind: Cow::Owned(us.kind.into_owned()),
            registry: us.registry.map(|v| Cow::Owned(v.into_owned())),
            package: package.map(Cow::Owned),
        }
    }
}

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

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Invalid JSON from client: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Invalid body")]
    MetadataParse,
    #[error("expected a valid crate name to start with a letter, contain only letters, numbers, hyphens, or underscores and have at most 64 characters ")]
    InvalidCrateName,
    #[error("Failed to push crate file to storage: {0}")]
    File(#[from] Box<chartered_fs::Error>),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::JsonParse(_) | Self::MetadataParse | Self::InvalidCrateName => {
                StatusCode::BAD_REQUEST
            }
            Self::File(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

define_error_response!(Error);
