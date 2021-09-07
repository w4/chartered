table! {
    crate_versions (id) {
        id -> Integer,
        crate_id -> Integer,
        version -> Text,
        filesystem_object -> Text,
        yanked -> Bool,
        checksum -> Text,
    }
}

table! {
    crates (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    user_api_keys (id) {
        id -> Integer,
        user_id -> Integer,
        api_key -> Text,
    }
}

table! {
    user_crate_permissions (id) {
        id -> Integer,
        user_id -> Integer,
        crate_id -> Integer,
        permissions -> Integer,
    }
}

table! {
    user_ssh_keys (id) {
        id -> Integer,
        user_id -> Integer,
        ssh_key -> Binary,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
    }
}

joinable!(crate_versions -> crates (crate_id));
joinable!(user_api_keys -> users (user_id));
joinable!(user_crate_permissions -> crates (crate_id));
joinable!(user_crate_permissions -> users (user_id));
joinable!(user_ssh_keys -> users (user_id));

allow_tables_to_appear_in_same_query!(
    crate_versions,
    crates,
    user_api_keys,
    user_crate_permissions,
    user_ssh_keys,
    users,
);
