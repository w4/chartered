use async_trait::async_trait;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct FileReference(uuid::Uuid);

impl std::fmt::Display for FileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait FileSystem {
    async fn read(&self, file_ref: FileReference) -> Result<Vec<u8>, std::io::Error>;
    async fn write(&self, data: &[u8]) -> Result<FileReference, std::io::Error>;
}

pub struct Local;

#[async_trait]
impl FileSystem for Local {
    async fn read(&self, file_ref: FileReference) -> Result<Vec<u8>, std::io::Error> {
        let mut file = File::open(format!("/tmp/{}", file_ref.0)).await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        Ok(contents)
    }

    async fn write(&self, data: &[u8]) -> Result<FileReference, std::io::Error> {
        let uuid = uuid::Uuid::new_v4();

        let mut file = File::create(format!("/tmp/{}", uuid)).await?;
        file.write_all(data).await?;

        Ok(FileReference(uuid))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
