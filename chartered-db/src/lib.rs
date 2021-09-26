#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)] // `sql_function` fails this check

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
pub mod organisations;
pub mod permissions;
pub mod schema;
pub mod users;
pub mod uuid;

#[macro_use]
extern crate diesel;

use diesel::{
    expression::{grouped::Grouped, AsExpression, Expression},
    r2d2::{ConnectionManager, Pool},
    sql_types::{Integer, Nullable},
};
use diesel_logger::LoggingConnection;
use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;

pub type ConnectionPool = Arc<Pool<ConnectionManager<LoggingConnection<diesel::SqliteConnection>>>>;
pub type Result<T> = std::result::Result<T, Error>;

pub fn init() -> Result<ConnectionPool> {
    Ok(Arc::new(Pool::new(ConnectionManager::new("chartered.db"))?))
}

#[derive(Error, Display, Debug)]
pub enum Error {
    /// Failed to initialise to database connection pool
    Connection(#[from] diesel::r2d2::PoolError),
    /// {0}
    Query(#[from] diesel::result::Error),
    /// Failed to complete query task
    TaskJoin(#[from] tokio::task::JoinError),
    /// Key parse failure: `{0}`
    KeyParse(#[from] thrussh_keys::Error),
    /// You don't have the {0:?} permission for this crate
    MissingCratePermission(crate::permissions::UserPermission),
    /// You don't have the {0:?} permission for this organisation
    MissingOrganisationPermission(crate::permissions::UserPermission),
    /// The requested crate does not exist
    MissingCrate,
    /// The requested organisation does not exist
    MissingOrganisation,
    /// Version {0} already exists for this crate
    VersionConflict(String),
}

impl Error {
    #[must_use]
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            Self::MissingCrate => http::StatusCode::NOT_FOUND,
            Self::MissingCratePermission(v) | Self::MissingOrganisationPermission(v)
                if v.contains(crate::permissions::UserPermission::VISIBLE) =>
            {
                http::StatusCode::NOT_FOUND
            }
            Self::MissingCratePermission(_) | Self::MissingOrganisationPermission(_) => {
                http::StatusCode::FORBIDDEN
            }
            Self::KeyParse(_) | Self::VersionConflict(_) => http::StatusCode::BAD_REQUEST,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

sql_function!(fn coalesce(x: Nullable<Integer>, y: Integer) -> Integer);

diesel_infix_operator!(BitwiseAnd, " & ", Integer);
diesel_infix_operator!(BitwiseOr, " | ", Integer);

trait BitwiseExpressionMethods: Expression<SqlType = Integer> + Sized {
    fn bitwise_and<T: AsExpression<Integer>>(
        self,
        other: T,
    ) -> Grouped<BitwiseAnd<Self, T::Expression>> {
        Grouped(BitwiseAnd::new(self.as_expression(), other.as_expression()))
    }

    fn bitwise_or<T: AsExpression<Integer>>(
        self,
        other: T,
    ) -> Grouped<BitwiseOr<Self, T::Expression>> {
        Grouped(BitwiseOr::new(self.as_expression(), other.as_expression()))
    }
}

impl<T: Expression<SqlType = Integer>> BitwiseExpressionMethods for T {}
