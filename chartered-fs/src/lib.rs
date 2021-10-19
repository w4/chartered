#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]

use std::{path::PathBuf, time::Duration};

use async_trait::async_trait;
use aws_sdk_s3::error::{GetObjectError, PutObjectError};
use aws_sdk_s3::{
    model::ObjectCannedAcl, presigning::config::PresigningConfig, ByteStream, SdkError,
};
use bytes::Bytes;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse filesystem uri: {0}")]
    UriParse(#[from] url::ParseError),
    #[error("unknown filesystem kind (expected `s3` or `file`)")]
    UnknownFileSystemKind,
    #[error("failed to insert object to s3: {0}")]
    S3Put(#[from] SdkError<PutObjectError>),
    #[error("failed to get object from s3: {0}")]
    S3Get(#[from] SdkError<GetObjectError>),
    #[error("i/o failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse uuid: {0}")]
    UuidParse(#[from] uuid::Error),
    #[error("path missing from uri")]
    MissingPath,
    #[error("host missing from uri")]
    MissingHost,
    #[error("bucket missing from uri")]
    MissingBucket,
    #[error("invalid aws presigning config: {0}")]
    AwsPresigningConfig(#[from] aws_sdk_s3::presigning::config::Error),
}

#[derive(Debug)]
pub enum FileSystem {
    S3(S3),
    Local(Local),
}

impl FileSystem {
    pub async fn from_str(s: &str) -> Result<Self, Error> {
        let uri = url::Url::parse(s)?;

        Ok(match uri.scheme() {
            "s3" => {
                let shared_config = aws_config::load_from_env().await;
                let client = aws_sdk_s3::Client::new(&shared_config);

                let mut path = uri.path_segments().ok_or(Error::MissingPath)?;

                Self::S3(S3 {
                    host: uri.host().ok_or(Error::MissingHost)?.to_string(),
                    bucket: path.next().ok_or(Error::MissingBucket)?.to_string(),
                    path: Itertools::intersperse(path, "/").collect(),
                    client,
                })
            }
            "file" => Self::Local(Local {
                path: uri.to_file_path().map_err(|()| Error::MissingPath)?,
            }),
            _ => return Err(Error::UnknownFileSystemKind),
        })
    }

    pub async fn read(&self, file_ref: FileReference) -> Result<FilePointer, Error> {
        match self {
            Self::S3(v) => v.read(file_ref).await,
            Self::Local(v) => v.read(file_ref).await,
        }
    }

    pub async fn write(&self, data: Bytes) -> Result<FileReference, Error> {
        match self {
            Self::S3(v) => v.write(data).await,
            Self::Local(v) => v.write(data).await,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileSystemKind {
    Local,
    S3,
}

impl std::fmt::Display for FileSystemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
            Self::S3 => f.write_str("s3"),
        }
    }
}

impl std::str::FromStr for FileSystemKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "local" => Ok(Self::Local),
            "s3" => Ok(Self::S3),
            _ => Err(Error::UnknownFileSystemKind),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReference {
    file_system: FileSystemKind,
    reference: uuid::Uuid,
}

impl std::fmt::Display for FileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file_system, self.reference)
    }
}

impl std::str::FromStr for FileReference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ':');
        let file_system = FileSystemKind::from_str(split.next().unwrap_or_default())?;
        let reference = uuid::Uuid::from_str(split.next().unwrap_or_default())?;
        Ok(FileReference {
            file_system,
            reference,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FilePointer {
    Content(Vec<u8>),
    Redirect(http::Uri),
}

#[async_trait]
pub trait FileSystemIo {
    const KIND: FileSystemKind;

    async fn read(&self, file_ref: FileReference) -> Result<FilePointer, Error>;
    async fn write(&self, data: Bytes) -> Result<FileReference, Error>;

    #[must_use]
    fn create_ref() -> FileReference {
        FileReference {
            file_system: Self::KIND,
            reference: uuid::Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct Local {
    pub path: PathBuf,
}

#[async_trait]
impl FileSystemIo for Local {
    const KIND: FileSystemKind = FileSystemKind::Local;

    async fn read(&self, file_ref: FileReference) -> Result<FilePointer, Error> {
        let path = self.path.join(file_ref.reference.to_string());
        let mut file = File::open(path).await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        Ok(FilePointer::Content(contents))
    }

    async fn write(&self, data: Bytes) -> Result<FileReference, Error> {
        let file_ref = Self::create_ref();
        let path = self.path.join(file_ref.reference.to_string());

        let mut file = File::create(path).await?;
        file.write_all(&data).await?;

        Ok(file_ref)
    }
}

#[derive(Debug)]
pub struct S3 {
    host: String,
    bucket: String,
    path: String,
    client: aws_sdk_s3::Client,
}

#[async_trait]
impl FileSystemIo for S3 {
    const KIND: FileSystemKind = FileSystemKind::S3;

    async fn read(&self, file_ref: FileReference) -> Result<FilePointer, Error> {
        Ok(FilePointer::Redirect(
            self.client
                .get_object()
                .key(format!("{}/{}", self.path, file_ref.reference.to_string()))
                .bucket(&self.bucket)
                .presigned(PresigningConfig::expires_in(Duration::from_secs(600))?)
                .await?
                .uri()
                .clone(),
        ))
    }

    async fn write(&self, data: Bytes) -> Result<FileReference, Error> {
        let file_ref = Self::create_ref();

        self.client
            .put_object()
            .key(format!("{}/{}", self.path, file_ref.reference.to_string()))
            .content_md5(format!("{:x}", md5::compute(&data)))
            .body(ByteStream::new(data.into()))
            .bucket(&self.bucket)
            .acl(ObjectCannedAcl::Private)
            .send()
            .await?;

        Ok(file_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::{FilePointer, FileSystem, FileSystemIo};
    use bytes::Bytes;

    #[tokio::test]
    #[allow(clippy::pedantic)]
    async fn parse_filesystem() {
        // assert!(matches!(
        //     FileSystem::from_str("s3://10.0.64.101:9000/my-bucket/my-location"),
        //     Ok(FileSystem::S3(inner)) if inner.host == "10.0.64.101" && inner.bucket == "my-bucket" && inner.path == "my-location"
        // ));
        assert!(matches!(
            FileSystem::from_str("file:///tmp/chartered").await,
            Ok(FileSystem::Local(inner)) if inner.path.to_str().unwrap() == "/tmp/chartered"
        ));
    }

    #[tokio::test]
    #[allow(clippy::pedantic)]
    async fn local() {
        let fs = super::Local {
            path: "/tmp".into(),
        };
        let file_ref = fs.write(Bytes::from_static(b"abcdef")).await.unwrap();
        assert_eq!(
            fs.read(file_ref).await.unwrap(),
            FilePointer::Content(Vec::from(b"abcdef".as_ref()))
        );
    }
}
