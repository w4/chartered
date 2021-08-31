#[macro_use]
extern crate diesel;

pub mod schema;

use std::sync::Arc;

use self::diesel::prelude::*;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Associations, Identifiable, Queryable,
};

use schema::crate_versions;
use schema::crates;

pub fn init() -> Arc<Pool<ConnectionManager<diesel::SqliteConnection>>> {
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

pub async fn get_crate_versions(
    conn: Arc<Pool<ConnectionManager<diesel::SqliteConnection>>>,
    crate_name: String,
) -> Vec<CrateVersion> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
