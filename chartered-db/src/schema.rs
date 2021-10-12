table! {
    crate_versions (id) {
        id -> Integer,
        crate_id -> Integer,
        version -> Text,
        filesystem_object -> Text,
        size -> Integer,
        yanked -> Bool,
        checksum -> Text,
        dependencies -> Binary,
        features -> Binary,
        links -> Nullable<Text>,
        user_id -> Integer,
        created_at -> Timestamp,
    }
}

table! {
    crates (id) {
        id -> Integer,
        name -> Text,
        organisation_id -> Integer,
        readme -> Nullable<Text>,
        description -> Nullable<Text>,
        repository -> Nullable<Text>,
        homepage -> Nullable<Text>,
        documentation -> Nullable<Text>,
        downloads -> Integer,
        created_at -> Timestamp,
    }
}

table! {
    organisations (id) {
        id -> Integer,
        uuid -> Binary,
        name -> Text,
        description -> Text,
        public -> Bool,
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
    user_organisation_permissions (id) {
        id -> Integer,
        user_id -> Integer,
        organisation_id -> Integer,
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
        name -> Nullable<Text>,
        nick -> Nullable<Text>,
        email -> Nullable<Text>,
        external_profile_url -> Nullable<Text>,
        picture_url -> Nullable<Text>,
    }
}

joinable!(crate_versions -> crates (crate_id));
joinable!(crate_versions -> users (user_id));
joinable!(crates -> organisations (organisation_id));
joinable!(user_crate_permissions -> crates (crate_id));
joinable!(user_crate_permissions -> users (user_id));
joinable!(user_organisation_permissions -> organisations (organisation_id));
joinable!(user_organisation_permissions -> users (user_id));
joinable!(user_sessions -> user_ssh_keys (user_ssh_key_id));
joinable!(user_sessions -> users (user_id));
joinable!(user_ssh_keys -> users (user_id));

allow_tables_to_appear_in_same_query!(
    crate_versions,
    crates,
    organisations,
    user_crate_permissions,
    user_organisation_permissions,
    user_sessions,
    user_ssh_keys,
    users,
);
