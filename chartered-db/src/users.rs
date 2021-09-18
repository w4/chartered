use super::{
    schema::{user_crate_permissions, user_sessions, user_ssh_keys, users},
    ConnectionPool, Result,
};
use bitflags::bitflags;
use diesel::{insert_into, prelude::*, Associations, Identifiable, Queryable};
use option_set::{option_set, OptionSet};
use rand::{thread_rng, Rng};
use std::sync::Arc;
use thrussh_keys::PublicKeyBase64;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl User {
    pub async fn search(
        conn: ConnectionPool,
        given_query: String,
        limit: i64,
    ) -> Result<Vec<User>> {
        use crate::schema::users::dsl::username;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::users::table
                .filter(username.like(format!("%{}%", given_query)))
                .limit(limit)
                .load(&conn)?)
        })
        .await?
    }

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

    pub async fn find_by_session_key(
        conn: ConnectionPool,
        given_session_key: String,
    ) -> Result<Option<User>> {
        use crate::schema::user_sessions::dsl::{expires_at, session_key};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(user_sessions::table
                .filter(
                    expires_at
                        .is_null()
                        .or(expires_at.gt(chrono::Utc::now().naive_utc())),
                )
                .filter(session_key.eq(given_session_key))
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
            (Some(_), Some(key)) | (Some(key), None) => key,
            _ => return Err(thrussh_keys::Error::CouldNotReadKey.into()),
        };

        let parsed_key = thrussh_keys::parse_public_key_base64(key)?;
        let parsed_name = split.next().unwrap_or("(none)").to_string();

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_ssh_keys::dsl::{name, ssh_key, user_id};

            let conn = conn.get()?;

            insert_into(crate::schema::user_ssh_keys::dsl::user_ssh_keys)
                .values((
                    name.eq(parsed_name),
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

    /// Get all the SSH keys for the user.
    pub async fn list_ssh_keys(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<UserSshKey>> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_ssh_keys::dsl::user_id;

            let conn = conn.get()?;

            let selected = crate::schema::user_ssh_keys::table
                .filter(user_id.eq(self.id))
                .inner_join(users::table)
                .select(user_ssh_keys::all_columns)
                .load(&conn)?;

            Ok(selected)
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
pub struct UserSession {
    pub id: i32,
    pub user_id: i32,
    pub session_key: String,
    pub user_ssh_key_id: Option<i32>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
}

impl UserSession {
    pub async fn generate(
        conn: ConnectionPool,
        given_user_id: i32,
        given_user_ssh_key_id: Option<i32>,
        given_expires_at: Option<chrono::NaiveDateTime>,
        given_user_agent: Option<String>,
        given_ip: Option<String>,
    ) -> Result<Self> {
        use crate::schema::user_sessions::dsl::{
            expires_at, ip, session_key, user_agent, user_id, user_sessions, user_ssh_key_id,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let generated_session_key: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(48)
                .map(char::from)
                .collect();

            insert_into(user_sessions)
                .values((
                    user_id.eq(given_user_id),
                    session_key.eq(&generated_session_key),
                    user_ssh_key_id.eq(given_user_ssh_key_id),
                    expires_at.eq(given_expires_at),
                    user_agent.eq(given_user_agent),
                    ip.eq(given_ip),
                ))
                .execute(&conn)?;

            Ok(crate::schema::user_sessions::table
                .filter(session_key.eq(generated_session_key))
                .get_result(&conn)?)
        })
        .await?
    }
}

option_set! {
    #[derive(FromSqlRow, AsExpression)]
    pub struct UserCratePermissionValue: Identity + i32 {
        const VISIBLE         = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const PUBLISH_VERSION = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        const YANK_VERSION    = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        const MANAGE_USERS    = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    }
}

impl UserCratePermissionValue {
    pub fn names() -> &'static [&'static str] {
        Self::NAMES
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
#[belongs_to(super::crates::Crate)]
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
    pub name: String,
    pub user_id: i32,
    pub ssh_key: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
    pub last_used_at: Option<chrono::NaiveDateTime>,
}

impl UserSshKey {
    /// Every SSH key should have a corresponding session so when the config is pulled from git we
    /// can return a key in there. The session might have, however, been compromised and removed
    /// using the Web UI/database/etc - this function will regenerate the key on next pull so
    /// there's no disruption in service.
    pub async fn get_or_insert_session(
        self: Arc<Self>,
        conn: ConnectionPool,
        ip: Option<String>,
    ) -> Result<UserSession> {
        use crate::schema::user_sessions::dsl::{expires_at, user_id};

        let res: Option<UserSession> = tokio::task::spawn_blocking({
            let conn = conn.clone();
            let this = self.clone();
            move || {
                let conn = conn.get()?;

                UserSession::belonging_to(&*this)
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
            UserSession::generate(conn, self.user_id, Some(self.id), None, None, ip).await
        }
    }

    /// Updates the last used time of this SSH key for reporting purposes in the
    /// dashboard.
    pub async fn update_last_used(self: Arc<Self>, conn: ConnectionPool) -> Result<()> {
        use crate::schema::user_ssh_keys::dsl::{id, last_used_at, user_ssh_keys};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            diesel::update(user_ssh_keys.filter(id.eq(self.id)))
                .set(last_used_at.eq(diesel::dsl::now))
                .execute(&conn)
                .map(|_| ())
                .map_err(Into::into)
        })
        .await?
    }

    /// Retrieves the key fingerprint, encoded in hex and separated in two character chunks
    /// with colons.
    pub fn fingerprint(&self) -> Result<String> {
        let key = thrussh_keys::key::parse_public_key(&self.ssh_key)?;

        let raw_hex = hex::encode(
            base64::decode(&key.fingerprint()).map_err(|_| thrussh_keys::Error::CouldNotReadKey)?,
        );
        let mut hex = String::with_capacity(raw_hex.len() + (raw_hex.len() / 2 - 1));

        for (i, c) in raw_hex.chars().enumerate() {
            if i != 0 && i % 2 == 0 {
                hex.push(':');
            }

            hex.push(c);
        }

        Ok(hex)
    }
}
