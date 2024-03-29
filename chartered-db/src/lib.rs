#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
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
                out: &mut diesel::serialize::Output<'_, W, B>,
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
pub mod server_private_key;
pub mod users;
pub mod uuid;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::{
    expression::{grouped::Grouped, AsExpression, Expression},
    r2d2::{ConnectionManager, Pool},
    sql_types::{Integer, Nullable},
};
use displaydoc::Display;
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "sqlite")]
pub type Connection = diesel_tracing::sqlite::InstrumentedSqliteConnection;

#[cfg(feature = "postgres")]
pub type Connection = diesel_tracing::pg::InstrumentedPgConnection;

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("Only one database backend must be enabled using --features [sqlite|postgres]");

#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!(
    "At least one database backend must be enabled using `--features [sqlite|postgres]`"
);
#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
pub type Connection = unimplemented!();

pub type ConnectionPool = Arc<Pool<ConnectionManager<Connection>>>;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "postgres")]
embed_migrations!("../migrations/postgres");

#[cfg(feature = "sqlite")]
embed_migrations!("../migrations/sqlite");

pub fn init(connection_uri: &str) -> Result<ConnectionPool> {
    let connection_uri = parse_connection_uri(connection_uri)?;
    let pool = Pool::new(ConnectionManager::new(connection_uri))?;

    embedded_migrations::run_with_output(&pool.get()?, &mut std::io::stdout())?;

    Ok(Arc::new(pool))
}

#[cfg(feature = "sqlite")]
pub fn parse_connection_uri(connection_uri: &str) -> Result<&str> {
    if connection_uri.starts_with("sqlite://") {
        Ok(connection_uri.trim_start_matches("sqlite://"))
    } else {
        Err(Error::SqliteConnectionUri)
    }
}

#[cfg(feature = "postgres")]
pub fn parse_connection_uri(connection_uri: &str) -> Result<&str> {
    if connection_uri.starts_with("postgres://") {
        Ok(connection_uri)
    } else {
        Err(Error::PostgresConnectionUri)
    }
}

#[derive(Error, Display, Debug)]
pub enum Error {
    /// connection_uri must be in the format `sqlite:///path/to/file.db` or `sqlite://:memory:`
    SqliteConnectionUri,
    /**
     * connection_uri must be a postgres connection uri as described in:
     * https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING
     */
    PostgresConnectionUri,
    /// Failed to initialise to database connection pool
    Connection(#[from] diesel::r2d2::PoolError),
    /// Failed to run migrations to bring database schema up-to-date: {0}
    MigrationError(#[from] diesel_migrations::RunMigrationsError),
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
    /// Username is already taken
    UsernameTaken,
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
