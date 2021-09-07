use axum::extract;
use bytes::Bytes;
use chartered_db::ConnectionPool;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

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
    body: Bytes,
) -> axum::response::Json<PublishCrateResponse> {
    use chartered_fs::FileSystem;
    use sha2::{Digest, Sha256};

    let (_, (metadata_bytes, crate_bytes)) = parse(body.as_ref()).unwrap();

    let metadata: Metadata = serde_json::from_slice(metadata_bytes).unwrap();

    let file_ref = chartered_fs::Local.write(crate_bytes).await.unwrap();

    let mut response = PublishCrateResponse::default();

    if let Err(e) = chartered_db::crates::publish_crate(
        db,
        metadata.name.to_string(),
        metadata.vers.to_string(),
        file_ref,
        hex::encode(Sha256::digest(crate_bytes)),
    )
    .await
    {
        // todo: should this be a normal http error?
        response.warnings.other.push(e.to_string());
    }

    axum::response::Json(response)
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
    name: &'a str,
    vers: &'a str,
    deps: Vec<MetadataDependency<'a>>,
    features: std::collections::HashMap<&'a str, Vec<&'a str>>,
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
    links: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
pub struct MetadataDependency<'a> {
    name: &'a str,
    version_req: &'a str,
    features: Vec<&'a str>,
    optional: bool,
    default_features: bool,
    target: Option<&'a str>,
    kind: &'a str, // 'dev', 'build', or 'normal'
    registry: &'a str,
    explicit_name_in_toml: Option<&'a str>,
}
