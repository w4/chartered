use super::{
    schema::{user_api_keys, user_crate_permissions, user_ssh_keys, users},
    ConnectionPool, Result,
};
use diesel::{insert_into, prelude::*, Associations, Identifiable, Queryable};
use rand::{thread_rng, Rng};
use std::{collections::HashMap, sync::Arc};
use thrussh_keys::PublicKeyBase64;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl User {
    pub async fn find_by_username(
        conn: ConnectionPool,
        given_username: String,
    ) -> Result<Option<User>> {
        use crate::schema::users::dsl::username;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::users::table
                .filter(username.eq(given_username))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn find_by_api_key(
        conn: ConnectionPool,
        given_api_key: String,
    ) -> Result<Option<User>> {
        use crate::schema::user_api_keys::dsl::{api_key, expires_at};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_api_keys::table
                .filter(
                    expires_at
                        .is_null()
                        .or(expires_at.gt(chrono::Utc::now().naive_utc())),
                )
                .filter(api_key.eq(given_api_key))
                .inner_join(users::table)
                .select((users::dsl::id, users::dsl::username))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn find_by_ssh_key(
        conn: ConnectionPool,
        given_ssh_key: Vec<u8>,
    ) -> Result<Option<(UserSshKey, User)>> {
        use crate::schema::user_ssh_keys::dsl::ssh_key;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_ssh_keys::table
                .filter(ssh_key.eq(given_ssh_key))
                .inner_join(users::table)
                .select((user_ssh_keys::all_columns, users::all_columns))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }

    /// Parses an ssh key from its `ssh-add -L` format (`ssh-ed25519 AAAAC3N...`) and
    /// inserts it to the database for the user.
    pub async fn insert_ssh_key(
        self: Arc<Self>,
        conn: ConnectionPool,
        ssh_key: &str,
    ) -> Result<()> {
        let mut split = ssh_key.split_whitespace();

        let key = match (split.next(), split.next()) {
            (Some(_), Some(key)) => key,
            (Some(key), None) => key,
            _ => return Err(thrussh_keys::Error::CouldNotReadKey.into()),
        };

        let parsed_key = thrussh_keys::parse_public_key_base64(key)?;

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_ssh_keys::dsl::{ssh_key, user_id};

            let conn = conn.get()?;

            insert_into(crate::schema::user_ssh_keys::dsl::user_ssh_keys)
                .values((
                    ssh_key.eq(parsed_key.public_key_bytes()),
                    user_id.eq(self.id),
                ))
                .execute(&conn)?;

            Ok(())
        })
        .await?
    }

    pub async fn delete_user_ssh_key_by_id(
        self: Arc<Self>,
        conn: ConnectionPool,
        ssh_key_id: i32,
    ) -> Result<bool> {
        use crate::schema::user_ssh_keys::dsl::{id, user_id, user_ssh_keys};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let rows = diesel::delete(
                user_ssh_keys
                    .filter(user_id.eq(self.id))
                    .filter(id.eq(ssh_key_id)),
            )
            .execute(&conn)?;

            Ok(rows > 0)
        })
        .await?
    }

    /// Get all the SSH keys for the user, returned as `id:fingerprint`.
    pub async fn list_ssh_keys(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<HashMap<i32, String>> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_ssh_keys::dsl::user_id;

            let conn = conn.get()?;

            let selected: Vec<(i32, Vec<u8>)> = crate::schema::user_ssh_keys::table
                .filter(user_id.eq(self.id))
                .inner_join(users::table)
                .select((user_ssh_keys::id, user_ssh_keys::ssh_key))
                .load(&conn)?;

            Ok(selected
                .into_iter()
                .map(|(id, key)| {
                    (
                        id,
                        thrussh_keys::key::parse_public_key(&key)
                            .map(|v| v.fingerprint())
                            .unwrap_or_else(|e| format!("INVALID: {}", e)),
                    )
                })
                .collect())
        })
        .await?
    }

    pub async fn accessible_crates(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Vec<(UserCratePermissionValue, crate::crates::Crate)>> {
        use crate::schema::crates;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(UserCratePermission::belonging_to(&*self)
                .inner_join(crate::schema::crates::table)
                .select((user_crate_permissions::permissions, crates::all_columns))
                .load(&conn)?)
        })
        .await?
    }

    pub async fn get_crate_permissions(
        self: Arc<Self>,
        conn: ConnectionPool,
        crate_id: i32,
    ) -> Result<UserCratePermissionValue> {
        Ok(UserCratePermission::find(conn, self.id, crate_id)
            .await?
            .unwrap_or_default()
            .permissions)
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
#[belongs_to(UserSshKey)]
pub struct UserApiKey {
    pub id: i32,
    pub user_id: i32,
    pub api_key: String,
    pub user_ssh_key_id: Option<i32>,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

impl UserApiKey {
    pub async fn generate(
        conn: ConnectionPool,
        given_user_id: i32,
        given_user_ssh_key_id: Option<i32>,
        given_expires_at: Option<chrono::NaiveDateTime>,
    ) -> Result<UserApiKey> {
        use crate::schema::user_api_keys::dsl::{
            api_key, expires_at, user_api_keys, user_id, user_ssh_key_id,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let generated_api_key: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(48)
                .map(char::from)
                .collect();

            insert_into(user_api_keys)
                .values((
                    user_id.eq(given_user_id),
                    api_key.eq(&generated_api_key),
                    user_ssh_key_id.eq(given_user_ssh_key_id),
                    expires_at.eq(given_expires_at),
                ))
                .execute(&conn)?;

            Ok(crate::schema::user_api_keys::table
                .filter(api_key.eq(generated_api_key))
                .get_result(&conn)?)
        })
        .await?
    }
}

bitflags::bitflags! {
    #[derive(FromSqlRow, AsExpression, Default)]
    pub struct UserCratePermissionValue: i32 {
        const VISIBLE         = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const PUBLISH_VERSION = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        const YANK_VERSION    = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        const MANAGE_USERS    = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    }
}

impl<B: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Integer, B>
    for UserCratePermissionValue
where
    i32: diesel::deserialize::FromSql<diesel::sql_types::Integer, B>,
{
    fn from_sql(
        bytes: Option<&B::RawValue>,
    ) -> std::result::Result<UserCratePermissionValue, Box<dyn std::error::Error + Send + Sync>>
    {
        let val = i32::from_sql(bytes)?;
        Ok(UserCratePermissionValue::from_bits_truncate(val))
    }
}

#[derive(Identifiable, Queryable, Associations, Default, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserCratePermission {
    pub id: i32,
    pub user_id: i32,
    pub crate_id: i32,
    pub permissions: UserCratePermissionValue,
}

impl UserCratePermission {
    pub async fn find(
        conn: ConnectionPool,
        given_user_id: i32,
        given_crate_id: i32,
    ) -> Result<Option<UserCratePermission>> {
        use crate::schema::user_crate_permissions::dsl::{crate_id, user_id};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_crate_permissions::table
                .filter(user_id.eq(given_user_id))
                .filter(crate_id.eq(given_crate_id))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserSshKey {
    pub id: i32,
    pub user_id: i32,
    pub ssh_key: Vec<u8>,
}

impl UserSshKey {
    /// Every SSH key should have a corresponding API key so when the config is pulled from git we
    /// can return a key in there. The API key might have, however, been compromised and removed
    /// using the Web UI/database/etc - this function will regenerate the key on next pull so
    /// there's no disruption in service.
    pub async fn get_or_insert_api_key(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<UserApiKey> {
        use crate::schema::user_api_keys::dsl::{expires_at, user_id};

        let res: Option<UserApiKey> = tokio::task::spawn_blocking({
            let conn = conn.clone();
            let this = self.clone();
            move || {
                let conn = conn.get()?;

                UserApiKey::belonging_to(&*this)
                    .filter(
                        expires_at
                            .is_null()
                            .or(expires_at.gt(chrono::Utc::now().naive_utc())),
                    )
                    .filter(user_id.eq(this.user_id))
                    .get_result(&conn)
                    .optional()
                    .map_err(crate::Error::Query)
            }
        })
        .await??;

        if let Some(res) = res {
            Ok(res)
        } else {
            UserApiKey::generate(conn, self.user_id, Some(self.id), None).await
        }
    }
}
