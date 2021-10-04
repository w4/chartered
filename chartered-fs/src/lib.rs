#![deny(clippy::pedantic)]

use std::path::PathBuf;

use async_trait::async_trait;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug)]
pub enum FS {
    S3 {
        host: String,
        bucket: String,
        path: String,
    },
    Local {
        path: PathBuf,
    },
}

impl std::str::FromStr for FS {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = url::Url::parse(s)?;

        Ok(match uri.scheme() {
            "s3" => {
                let mut path = uri.path_segments().unwrap();

                Self::S3 {
                    host: uri.host().unwrap().to_string(),
                    bucket: path.next().unwrap().to_string(),
                    path: path.intersperse("/").collect(),
                }
            }
            "file" => {
                panic!("{:#?}", uri);
            }
            _ => panic!("na"),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileSystemKind {
    Local,
    S3,
}

impl std::fmt::Display for FileSystemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
            Self::S3 => f.write_str("s3"),
        }
    }
}

impl std::str::FromStr for FileSystemKind {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "unknown filesystemkind",
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReference {
    file_system: FileSystemKind,
    reference: uuid::Uuid,
}

impl std::fmt::Display for FileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.file_system, self.reference)
    }
}

impl std::str::FromStr for FileReference {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ':');
        let file_system = FileSystemKind::from_str(split.next().unwrap_or_default())?;
        let reference = uuid::Uuid::from_str(split.next().unwrap_or_default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(FileReference {
            file_system,
            reference,
        })
    }
}

#[async_trait]
pub trait FileSystem {
    const KIND: FileSystemKind;

    async fn read(&self, file_ref: FileReference) -> Result<Vec<u8>, std::io::Error>;
    async fn write(&self, data: &[u8]) -> Result<FileReference, std::io::Error>;

    #[must_use]
    fn create_ref() -> FileReference {
        FileReference {
            file_system: Self::KIND,
            reference: uuid::Uuid::new_v4(),
        }
    }
}

pub struct Local;

#[async_trait]
impl FileSystem for Local {
    const KIND: FileSystemKind = FileSystemKind::Local;

    async fn read(&self, file_ref: FileReference) -> Result<Vec<u8>, std::io::Error> {
        let mut file = File::open(format!("/tmp/{}", file_ref.reference)).await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        Ok(contents)
    }

    async fn write(&self, data: &[u8]) -> Result<FileReference, std::io::Error> {
        let file_ref = Self::create_ref();

        let mut file = File::create(format!("/tmp/{}", file_ref.reference)).await?;
        file.write_all(data).await?;

        Ok(file_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::{FileSystem, FS};
    use std::str::FromStr;

    #[tokio::test]
    #[allow(clippy::pedantic)]
    async fn parse_filesystem() {
        // panic!("{:#?}", FS::from_str("s3://10.0.64.101:9000/my-bucket/my-location"));
        FS::from_str("file:///tmp/chartered");
    }

    #[tokio::test]
    #[allow(clippy::pedantic)]
    async fn local() {
        let fs = super::Local;
        let file_ref = fs.write(b"abcdef").await.unwrap();
        assert_eq!(fs.read(file_ref).await.unwrap(), b"abcdef");
    }
}
