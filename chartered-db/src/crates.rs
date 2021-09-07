use super::{
    schema::{crate_versions, crates},
    ConnectionPool, Result,
};
use diesel::{
    insert_into, insert_or_ignore_into, prelude::*, Associations, Identifiable, Queryable,
};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

#[derive(Identifiable, Queryable, PartialEq, Eq, Hash, Debug)]
pub struct Crate {
    pub id: i32,
    pub name: String,
}

impl Crate {
    pub async fn all_with_versions(
        conn: ConnectionPool,
    ) -> Result<HashMap<Crate, Vec<CrateVersion>>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

            let crate_versions = crates::table
                .inner_join(crate_versions::table)
                .load(&conn)?;

            Ok(crate_versions.into_iter().into_grouping_map().collect())
        })
        .await?
    }

    pub async fn find_by_name(conn: ConnectionPool, crate_name: String) -> Result<Option<Self>> {
        use crate::schema::crates::dsl::*;

        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

            crates
                .filter(name.eq(crate_name))
                .first::<Crate>(&conn)
                .optional()
        })
        .await??)
    }

    pub async fn versions(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<CrateVersion>> {
        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

            CrateVersion::belonging_to(&*self).load::<CrateVersion>(&conn)
        })
        .await??)
    }

    pub async fn version(
        self: Arc<Self>,
        conn: ConnectionPool,
        crate_version: String,
    ) -> Result<Option<CrateVersion>> {
        use crate::schema::crate_versions::*;

        Ok(tokio::task::spawn_blocking(move || {
            let conn = conn.get().unwrap();

            CrateVersion::belonging_to(&*self)
                .filter(version.eq(crate_version))
                .get_result::<CrateVersion>(&conn)
                .optional()
        })
        .await??)
    }
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
