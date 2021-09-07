#![deny(clippy::pedantic)]
#![deny(clippy::pedantic)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum FileSystemKind {
    Local,
}

impl std::fmt::Display for FileSystemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
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
    use super::FileSystem;

    #[tokio::test]
    async fn local() {
        let fs = super::Local;
        let file_ref = fs.write(b"abcdef").await.unwrap();
        assert_eq!(fs.read(file_ref).await.unwrap(), b"abcdef");
    }
}
