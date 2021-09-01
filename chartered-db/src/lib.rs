pub mod schema;

#[macro_use]
extern crate diesel;

use diesel::{Associations, Identifiable, Queryable, insert_into, insert_or_ignore_into, prelude::*, r2d2::{ConnectionManager, Pool}};
use schema::{crate_versions, crates};
use std::sync::Arc;

pub type ConnectionPool = Arc<Pool<ConnectionManager<diesel::SqliteConnection>>>;

pub fn init() -> ConnectionPool {
    Arc::new(Pool::new(ConnectionManager::new("chartered.db")).unwrap())
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct Crate {
    id: i32,
    name: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Crate)]
pub struct CrateVersion {
    id: i32,
    crate_id: i32,
    version: String,
    filesystem_object: String,
    yanked: bool,
}

pub async fn get_crate_versions(conn: ConnectionPool, crate_name: String) -> Vec<CrateVersion> {
    use crate::schema::crates::dsl::*;

    tokio::task::spawn_blocking(move || {
        let conn = conn.get().unwrap();

        let selected_crate = crates
            .filter(name.eq(crate_name))
            .first::<Crate>(&conn)
            .expect("no crate");
        let selected_crate_versions = CrateVersion::belonging_to(&selected_crate)
            .load::<CrateVersion>(&conn)
            .expect("no crate versions");

        selected_crate_versions
    })
    .await
    .unwrap()
}

pub async fn publish_crate(
    conn: ConnectionPool,
    crate_name: String,
    version_string: String,
    file_identifier: chartered_fs::FileReference,
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
