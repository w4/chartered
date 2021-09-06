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

joinable!(crate_versions -> crates (crate_id));

allow_tables_to_appear_in_same_query!(
    crate_versions,
    crates,
);
