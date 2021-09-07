use super::{
    schema::{user_api_keys, user_crate_permissions, user_ssh_keys, users},
    ConnectionPool, Result,
};
use diesel::{prelude::*, Associations, Identifiable, Queryable};
use std::sync::Arc;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    id: i32,
    username: String,
}

impl User {
    pub async fn find_by_api_key(
        conn: ConnectionPool,
        given_api_key: String,
    ) -> Result<Option<User>> {
        use crate::schema::user_api_keys::dsl::*;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

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
        use crate::schema::user_ssh_keys::dsl::*;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

            Ok(crate::schema::user_ssh_keys::table
                .filter(ssh_key.eq(given_ssh_key))
                .inner_join(users::table)
                .select((users::dsl::id, users::dsl::username))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserApiKey {
    id: i32,
    user_id: i32,
    api_key: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserCratePermission {
    id: i32,
    user_id: i32,
    crate_id: i32,
    permissions: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
pub struct UserSshKey {
    id: i32,
    user_id: i32,
    ssh_key: Vec<u8>,
}
