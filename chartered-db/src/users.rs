use super::schema::{user_api_keys, user_crate_permissions, user_ssh_keys, users};
use diesel::{Associations, Identifiable, Queryable};

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct User {
    id: i32,
    username: String,
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
