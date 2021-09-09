#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod crates;
pub mod schema;
pub mod users;

#[macro_use]
extern crate diesel;

use diesel::{
    expression::{AsExpression, Expression},
    r2d2::{ConnectionManager, Pool},
};
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

diesel_infix_operator!(BitwiseAnd, " & ", diesel::sql_types::Integer);

trait BitwiseExpressionMethods: Expression<SqlType = diesel::sql_types::Integer> + Sized {
    fn bitwise_and<T: AsExpression<diesel::sql_types::Integer>>(
        self,
        other: T,
    ) -> BitwiseAnd<Self, T::Expression> {
        BitwiseAnd::new(self.as_expression(), other.as_expression())
    }
}

impl<T: Expression<SqlType = diesel::sql_types::Integer>> BitwiseExpressionMethods for T {}
