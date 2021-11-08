use super::{
    crates::UserCratePermission,
    permissions::UserPermission,
    schema::{user_crate_permissions, user_sessions, user_ssh_keys, users},
    uuid::SqlUuid,
    ConnectionPool, Error, Result,
};
use diesel::result::DatabaseErrorKind;
use diesel::{
    insert_into, prelude::*, result::Error as DieselError, Associations, Identifiable, Queryable,
};
use rand::{thread_rng, Rng};
use std::sync::Arc;
use thrussh_keys::PublicKeyBase64;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub id: i32,
    pub uuid: SqlUuid,
    pub username: String,
    pub password: Option<String>,
    pub name: Option<String>,
    pub nick: Option<String>,
    pub email: Option<String>,
    pub external_profile_url: Option<String>,
    pub picture_url: Option<String>,
}

impl User {
    pub async fn search(
        conn: ConnectionPool,
        given_query: String,
        limit: i64,
    ) -> Result<Vec<User>> {
        use crate::schema::users::dsl::{name, nick, username};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let query = format!("%{}%", given_query);

            Ok(crate::schema::users::table
                .filter(
                    username
                        .like(&query)
                        .or(name.like(&query))
                        .or(nick.like(&query)),
                )
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

    pub async fn find_by_uuid(
        conn: ConnectionPool,
        given_uuid: uuid::Uuid,
    ) -> Result<Option<User>> {
        use crate::schema::users::dsl::uuid;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::users::table
                .filter(uuid.eq(SqlUuid(given_uuid)))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn find_by_session_key(
        conn: ConnectionPool,
        given_session_key: String,
    ) -> Result<Option<(UserSession, User)>> {
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
                .select((user_sessions::all_columns, users::all_columns))
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

    /// Lookup the user in the database by username, or create the user if it doesn't yet
    /// exist. The user will be created with no password so it cannot be logged into using
    /// standard `password` auth, and must be logged into using OAuth.
    pub async fn find_or_create(
        conn: ConnectionPool,
        given_username: String,
        given_name: Option<String>,
        given_nick: Option<String>,
        given_email: Option<String>,
        given_external_profile_url: Option<reqwest::Url>,
        given_picture_url: Option<reqwest::Url>,
    ) -> Result<User> {
        use crate::schema::users::dsl::{
            email, external_profile_url, name, nick, picture_url, username, uuid,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let user: Option<User> = crate::schema::users::table
                .filter(username.eq(&given_username))
                .get_result(&conn)
                .optional()?;

            if let Some(user) = user {
                return Ok(user);
            }

            diesel::insert_into(users::table)
                .values((
                    username.eq(&given_username),
                    uuid.eq(SqlUuid::random()),
                    name.eq(&given_name),
                    nick.eq(&given_nick),
                    email.eq(&given_email),
                    external_profile_url.eq(given_external_profile_url.map(|v| v.to_string())),
                    picture_url.eq(given_picture_url.map(|v| v.to_string())),
                ))
                .execute(&conn)?;

            Ok(crate::schema::users::table
                .filter(username.eq(given_username))
                .get_result(&conn)?)
        })
        .await?
    }

    pub async fn register(
        conn: ConnectionPool,
        username: String,
        password_hash: String,
    ) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let res = diesel::insert_into(users::table)
                .values((
                    users::username.eq(&username),
                    users::uuid.eq(SqlUuid::random()),
                    users::password.eq(&password_hash),
                ))
                .execute(&conn);

            match res {
                Ok(_) => Ok(()),
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    Err(Error::UsernameTaken)
                }
                Err(e) => Err(e.into()),
            }
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
            use crate::schema::user_ssh_keys::dsl::{name, ssh_key, user_id, uuid};

            let conn = conn.get()?;

            insert_into(crate::schema::user_ssh_keys::dsl::user_ssh_keys)
                .values((
                    uuid.eq(SqlUuid::random()),
                    name.eq(parsed_name),
                    ssh_key.eq(parsed_key.public_key_bytes()),
                    user_id.eq(self.id),
                ))
                .execute(&conn)?;

            Ok(())
        })
        .await?
    }

    pub async fn delete_user_ssh_key_by_uuid(
        self: Arc<Self>,
        conn: ConnectionPool,
        ssh_key_id: uuid::Uuid,
    ) -> Result<bool> {
        use crate::schema::user_ssh_keys::dsl::{user_id, user_ssh_keys, uuid};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let rows = diesel::delete(
                user_ssh_keys
                    .filter(user_id.eq(self.id))
                    .filter(uuid.eq(SqlUuid(ssh_key_id))),
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
    ) -> Result<Vec<(UserPermission, crate::crates::Crate)>> {
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
    ) -> Result<UserPermission> {
        Ok(UserCratePermission::find(conn, self.id, crate_id)
            .await?
            .unwrap_or_default()
            .permissions)
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        self.nick
            .as_ref()
            .or_else(|| self.name.as_ref())
            .unwrap_or(&self.username)
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
    pub uuid: SqlUuid,
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
            expires_at, ip, session_key, user_agent, user_id, user_sessions, user_ssh_key_id, uuid,
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
                    uuid.eq(SqlUuid::random()),
                ))
                .execute(&conn)?;

            Ok(crate::schema::user_sessions::table
                .filter(session_key.eq(generated_session_key))
                .get_result(&conn)?)
        })
        .await?
    }

    pub async fn list(
        conn: ConnectionPool,
        given_user_id: i32,
    ) -> Result<Vec<(Self, Option<UserSshKey>)>> {
        use crate::schema::user_sessions::dsl::{expires_at, user_id};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_sessions::table
                .filter(
                    expires_at
                        .is_null()
                        .or(expires_at.gt(chrono::Utc::now().naive_utc())),
                )
                .filter(user_id.eq(given_user_id))
                .left_join(user_ssh_keys::table)
                .load(&conn)?)
        })
        .await?
    }

    pub async fn delete(self: Arc<Self>, conn: ConnectionPool) -> Result<bool> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let res = diesel::delete(user_sessions::table)
                .filter(user_sessions::id.eq(self.id))
                .execute(&conn)?;

            Ok(res > 0)
        })
        .await?
    }

    pub async fn delete_by_uuid(conn: ConnectionPool, uuid: uuid::Uuid) -> Result<bool> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let res = diesel::delete(user_sessions::table)
                .filter(user_sessions::uuid.eq(SqlUuid(uuid)))
                .execute(&conn)?;

            Ok(res > 0)
        })
        .await?
    }

    pub async fn extend(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_expires_at: chrono::NaiveDateTime,
    ) -> Result<bool> {
        use crate::schema::user_sessions::dsl::expires_at;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let res = diesel::update(user_sessions::table)
                .filter(user_sessions::id.eq(self.id))
                .set(expires_at.eq(given_expires_at))
                .execute(&conn)?;

            Ok(res > 0)
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserSshKey {
    pub id: i32,
    pub uuid: SqlUuid,
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
