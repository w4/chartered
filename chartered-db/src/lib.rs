pub mod schema;

#[macro_use]
extern crate diesel;

use diesel::{
    insert_into, insert_or_ignore_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    Associations, Identifiable, Queryable,
};
use itertools::Itertools;
use schema::{crate_versions, crates};
use std::{collections::HashMap, sync::Arc};

pub type ConnectionPool = Arc<Pool<ConnectionManager<diesel::SqliteConnection>>>;

pub fn init() -> ConnectionPool {
    Arc::new(Pool::new(ConnectionManager::new("chartered.db")).unwrap())
}

#[derive(Identifiable, Queryable, PartialEq, Eq, Hash, Debug)]
pub struct Crate {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Crate)]
pub struct CrateVersion {
    pub id: i32,
    pub crate_id: i32,
    pub version: String,
    pub filesystem_object: String,
    pub yanked: bool,
    pub checksum: String,
}

pub async fn get_crate_versions(conn: ConnectionPool, crate_name: String) -> Vec<CrateVersion> {
    use crate::schema::crates::dsl::*;

    tokio::task::spawn_blocking(move || {
        let conn = conn.get().unwrap();

        let selected_crate = crates
            .filter(name.eq(crate_name))
            .first::<Crate>(&conn)
            .expect("no crate");

        CrateVersion::belonging_to(&selected_crate)
            .load::<CrateVersion>(&conn)
            .expect("no crate versions")
    })
    .await
    .unwrap()
}

pub async fn get_specific_crate_version(
    conn: ConnectionPool,
    crate_name: String,
    crate_version: String,
) -> Option<CrateVersion> {
    use crate::schema::{crate_versions::dsl::*, crates::dsl::*};

    tokio::task::spawn_blocking(move || {
        let conn = conn.get().unwrap();

        let selected_crate = crates
            .filter(name.eq(crate_name))
            .first::<Crate>(&conn)
            .expect("no crate");

        CrateVersion::belonging_to(&selected_crate)
            .filter(version.eq(crate_version))
            .get_result::<CrateVersion>(&conn)
            .optional()
            .expect("no crate version")
    })
    .await
    .unwrap()
}

pub async fn get_crates(conn: ConnectionPool) -> HashMap<Crate, Vec<CrateVersion>> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.get().unwrap();

        let crate_versions = crates::table
            .inner_join(crate_versions::table)
            .load(&conn)
            .unwrap();

        crate_versions.into_iter().into_grouping_map().collect()
    })
    .await
    .unwrap()
}

pub async fn publish_crate(
    conn: ConnectionPool,
    crate_name: String,
    version_string: String,
    file_identifier: chartered_fs::FileReference,
    file_checksum: String,
) {
    use crate::schema::{crate_versions::dsl::*, crates::dsl::*};

    tokio::task::spawn_blocking(move || {
        let conn = conn.get().unwrap();

        insert_or_ignore_into(crates)
            .values(name.eq(&crate_name))
            .execute(&conn)
            .unwrap();

        let selected_crate = crates
            .filter(name.eq(crate_name))
            .first::<Crate>(&conn)
            .unwrap();

        insert_into(crate_versions)
            .values((
                crate_id.eq(selected_crate.id),
                version.eq(version_string),
                filesystem_object.eq(file_identifier.to_string()),
                checksum.eq(file_checksum),
            ))
            .execute(&conn)
            .unwrap();
    })
    .await
    .unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
