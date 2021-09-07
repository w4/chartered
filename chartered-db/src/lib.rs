pub mod crates;
pub mod schema;
pub mod users;

#[macro_use]
extern crate diesel;

use diesel::r2d2::{ConnectionManager, Pool};
use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;

pub type ConnectionPool = Arc<Pool<ConnectionManager<diesel::SqliteConnection>>>;
pub type Result<T> = std::result::Result<T, Error>;

pub fn init() -> Result<ConnectionPool> {
    Ok(Arc::new(Pool::new(ConnectionManager::new("chartered.db"))?))
}

#[derive(Error, Display, Debug)]
pub enum Error {
    /// Failed to initialise to database connection pool: `{0}`
    Connection(#[from] diesel::r2d2::PoolError),
    /// Failed to run query: `{0}`
    Query(#[from] diesel::result::Error),
    /// Failed to complete query task: `{0}`
    TaskJoin(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
