table! {
    crate_versions (id) {
        id -> Integer,
        crate_id -> Integer,
        version -> Text,
        filesystem_object -> Text,
        yanked -> Bool,
        checksum -> Text,
        dependencies -> Binary,
        features -> Binary,
        links -> Nullable<Text>,
    }
}

table! {
    crates (id) {
        id -> Integer,
        name -> Text,
        readme -> Nullable<Text>,
        description -> Nullable<Text>,
        repository -> Nullable<Text>,
        homepage -> Nullable<Text>,
        documentation -> Nullable<Text>,
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
    user_sessions (id) {
        id -> Integer,
        user_id -> Integer,
        session_key -> Text,
        user_ssh_key_id -> Nullable<Integer>,
        expires_at -> Nullable<Timestamp>,
        user_agent -> Nullable<Text>,
        ip -> Nullable<Text>,
    }
}

table! {
    user_ssh_keys (id) {
        id -> Integer,
        uuid -> Binary,
        name -> Text,
        user_id -> Integer,
        ssh_key -> Binary,
        created_at -> Timestamp,
        last_used_at -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Integer,
        uuid -> Binary,
        username -> Text,
    }
}

joinable!(crate_versions -> crates (crate_id));
joinable!(user_crate_permissions -> crates (crate_id));
joinable!(user_crate_permissions -> users (user_id));
joinable!(user_sessions -> user_ssh_keys (user_ssh_key_id));
joinable!(user_sessions -> users (user_id));
joinable!(user_ssh_keys -> users (user_id));

allow_tables_to_appear_in_same_query!(
    crate_versions,
    crates,
    user_crate_permissions,
    user_sessions,
    user_ssh_keys,
    users,
);
