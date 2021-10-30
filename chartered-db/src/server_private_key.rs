use crate::{schema::server_private_keys, ConnectionPool, Error as CrateError};
use diesel::{
    insert_into, prelude::*, result::DatabaseErrorKind, result::Error as DieselError, Associations,
    Identifiable, Queryable,
};
use displaydoc::Display;
use thiserror::Error;
use thrussh_keys::key::{self, KeyPair};
use tracing::{info, info_span};

/// Represents a single SSH private key for the server.
///
/// We store these in the database as we need consistency across all hosts that may be
/// running `chartered-git` so clients don't get MITM warnings.
#[derive(Identifiable, Queryable, Associations, Default, PartialEq, Eq, Hash, Debug)]
pub struct ServerPrivateKey {
    pub id: i32,
    pub ssh_key_type: String,
    pub ssh_private_key: Vec<u8>,
}

impl ServerPrivateKey {
    /// Creates all the required keys for the server (currently just ed25519), ignoring any
    /// UNIQUE constraint errors.
    pub async fn create_if_not_exists(conn: ConnectionPool) -> Result<(), PrivateKeyError> {
        tokio::task::spawn_blocking(move || {
            info_span!("create_if_not_exists").in_scope(move || {
                let conn = conn.get()?;

                // diesel-tracing prints the UNIQUE constraint error even though we ignore it
                info!(
                    "Generating an ed25519 key if it doesn't already exist, UNIQUE constraint \
                        errors here can be safely ignored."
                );

                let ed25519_key = key::KeyPair::generate_ed25519()
                    .ok_or(PrivateKeyError::KeyGenerate("ed25519"))?;

                let res = insert_into(server_private_keys::table)
                    .values((
                        server_private_keys::ssh_key_type.eq("ed25519"),
                        server_private_keys::ssh_private_key.eq(private_key_bytes(&ed25519_key)?),
                    ))
                    .execute(&conn);

                match res {
                    Ok(_)
                    | Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            })
        })
        .await?
    }

    pub async fn fetch_all(conn: ConnectionPool) -> Result<Vec<Self>, CrateError> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            server_private_keys::table
                .select(server_private_keys::all_columns)
                .load(&conn)
                .map_err(Into::into)
        })
        .await?
    }

    /// Converts this `ServerPrivateKey` to thrussh's `KeyPair` type.
    pub fn into_private_key(self) -> Result<KeyPair, PrivateKeyError> {
        match self.ssh_key_type.as_str() {
            "ed25519" => {
                if self.ssh_private_key.len() != 64 {
                    return Err(PrivateKeyError::InvalidPrivateKey(self.ssh_key_type));
                }

                let mut private_key = [0_u8; 64];
                private_key.copy_from_slice(&self.ssh_private_key);

                Ok(KeyPair::Ed25519(key::ed25519::SecretKey {
                    key: private_key,
                }))
            }
            _ => Err(PrivateKeyError::UnknownPrivateKeyType(self.ssh_key_type)),
        }
    }
}

/// Grabs the private key bytes out of a `thrussh_keys::KeyPair`.
fn private_key_bytes(key: &KeyPair) -> Result<Vec<u8>, PrivateKeyError> {
    #[allow(unreachable_patterns)]
    match key {
        KeyPair::Ed25519(key::ed25519::SecretKey { key }) => Ok(key.to_vec()),
        _ => Err(PrivateKeyError::KeyGenerate(key.name())),
    }
}

#[derive(Error, Display, Debug)]
pub enum PrivateKeyError {
    /// Failed to generate {0} private key
    KeyGenerate(&'static str),
    /// Invalid {0} private key
    InvalidPrivateKey(String),
    /// Found {0} private key but chartered cannot handle this type
    UnknownPrivateKeyType(String),
    /// Failed to initialise to database connection pool
    Connection(#[from] diesel::r2d2::PoolError),
    /// Failed to complete query task
    TaskJoin(#[from] tokio::task::JoinError),
    /// {0}
    Query(#[from] DieselError),
}
