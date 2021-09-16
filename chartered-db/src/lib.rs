#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

macro_rules! derive_diesel_json {
    ($typ:ident$(<$lt:lifetime>)?) => {
        impl<$($lt, )?B: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::Blob, B> for $typ$(<$lt>)?
        where
            Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Blob, B>,
        {
            fn from_sql(bytes: Option<&B::RawValue>) -> diesel::deserialize::Result<Self> {
                let bytes = <Vec<u8>>::from_sql(bytes)?; // todo: we either have to allocate or deal with a raw pointer...
                serde_json::from_slice(&bytes).map_err(|_| "Invalid Json".into())
            }
        }

        impl<$($lt, )?B: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::Blob, B>
            for $typ$(<$lt>)?
        {
            fn to_sql<W: std::io::Write>(
                &self,
                out: &mut diesel::serialize::Output<W, B>,
            ) -> diesel::serialize::Result {
                serde_json::to_writer(out, self)
                    .map(|_| diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }
    };
}

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
    /// Key parse failure: `{0}`
    KeyParse(#[from] thrussh_keys::Error),
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
