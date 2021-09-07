use super::{
    schema::{user_api_keys, user_crate_permissions, user_ssh_keys, users},
    ConnectionPool, Result,
};
use diesel::{prelude::*, Associations, Identifiable, Queryable};
use std::sync::Arc;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl User {
    pub async fn find_by_api_key(
        conn: ConnectionPool,
        given_api_key: String,
    ) -> Result<Option<User>> {
        use crate::schema::user_api_keys::dsl::api_key;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_api_keys::table
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
    ) -> Result<Option<User>> {
        use crate::schema::user_ssh_keys::dsl::ssh_key;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_ssh_keys::table
                .filter(ssh_key.eq(given_ssh_key))
                .inner_join(users::table)
                .select((users::dsl::id, users::dsl::username))
                .get_result(&conn)
                .optional()?)
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
                .select((
                    user_crate_permissions::permissions,
                    (crates::dsl::id, crates::dsl::name),
                ))
                .load(&conn)?)
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserApiKey {
    pub id: i32,
    pub user_id: i32,
    pub api_key: String,
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
