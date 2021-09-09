use super::{
    schema::{crate_versions, crates},
    BitwiseExpressionMethods, ConnectionPool, Result,
};
use diesel::{insert_into, prelude::*, Associations, Identifiable, Queryable};
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
            let conn = conn.get()?;

            let crate_versions = crates::table
                .inner_join(crate_versions::table)
                .load(&conn)?;

            Ok(crate_versions.into_iter().into_grouping_map().collect())
        })
        .await?
    }

    pub async fn find_by_name(conn: ConnectionPool, crate_name: String) -> Result<Option<Self>> {
        use crate::schema::crates::dsl::{crates, name};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crates
                .filter(name.eq(crate_name))
                .first::<Crate>(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn versions(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<CrateVersion>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(CrateVersion::belonging_to(&*self).load::<CrateVersion>(&conn)?)
        })
        .await?
    }

    pub async fn version(
        self: Arc<Self>,
        conn: ConnectionPool,
        crate_version: String,
    ) -> Result<Option<CrateVersion>> {
        use crate::schema::crate_versions::version;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(CrateVersion::belonging_to(&*self)
                .filter(version.eq(crate_version))
                .get_result::<CrateVersion>(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn owners(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<crate::users::User>> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::{
                dsl::permissions, dsl::user_crate_permissions,
            };

            let conn = conn.get()?;

            Ok(user_crate_permissions
                .filter(
                    permissions
                        .bitwise_and(crate::users::UserCratePermissionValue::MANAGE_USERS.bits())
                        .ne(0),
                )
                .inner_join(crate::schema::users::dsl::users)
                .select(crate::schema::users::all_columns)
                .load::<crate::users::User>(&conn)?)
        })
        .await?
    }

    pub async fn publish_version(
        self: Arc<Self>,
        conn: ConnectionPool,
        version_string: String,
        file_identifier: chartered_fs::FileReference,
        file_checksum: String,
    ) -> Result<()> {
        use crate::schema::crate_versions::dsl::{
            checksum, crate_id, crate_versions, filesystem_object, version,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            insert_into(crate_versions)
                .values((
                    crate_id.eq(self.id),
                    version.eq(version_string),
                    filesystem_object.eq(file_identifier.to_string()),
                    checksum.eq(file_checksum),
                ))
                .execute(&conn)?;

            Ok(())
        })
        .await?
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
