use crate::users::UserCratePermission;

use super::{
    schema::{crate_versions, crates},
    BitwiseExpressionMethods, ConnectionPool, Result,
};
use diesel::{insert_into, prelude::*, Associations, Identifiable, Queryable};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Identifiable, Queryable, PartialEq, Eq, Hash, Debug)]
pub struct Crate {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Crate)]
pub struct CrateVersion<'a> {
    pub id: i32,
    pub crate_id: i32,
    pub version: String,
    pub filesystem_object: String,
    pub yanked: bool,
    // TODO: readme should be versioned in the db so we can just pass a reference rather
    // than this massive blob for each version - or just update the readme for the whole
    // crate and don't version it at all? we also need tags, etc too in a pivot table
    pub readme: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub checksum: String,
    pub dependencies: CrateDependencies<'a>,
    pub features: CrateFeatures,
    pub links: Option<String>,
}

impl<'a> CrateVersion<'a> {
    #[must_use]
    pub fn into_cargo_format(
        self,
        crate_: &'a Crate,
    ) -> (
        chartered_types::cargo::CrateVersion<'a>,
        chartered_types::cargo::CrateVersionMetadata,
    ) {
        (
            chartered_types::cargo::CrateVersion {
                name: crate_.name.as_str().into(),
                vers: self.version.into(),
                deps: self.dependencies.0,
                features: self.features.0,
                links: self.links.map(Into::into),
            },
            chartered_types::cargo::CrateVersionMetadata {
                description: self.description,
                readme: self.readme,
                repository: self.repository,
                homepage: self.homepage,
                documentation: self.documentation,
            },
        )
    }
}

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Debug, Clone, PartialEq, Eq)]
#[sql_type = "diesel::sql_types::Blob"]
pub struct CrateDependencies<'a>(pub Vec<chartered_types::cargo::CrateDependency<'a>>);

derive_diesel_json!(CrateDependencies<'a>);

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Debug, Clone, PartialEq, Eq)]
#[sql_type = "diesel::sql_types::Blob"]
pub struct CrateFeatures(pub chartered_types::cargo::CrateFeatures);

derive_diesel_json!(CrateFeatures);

impl Crate {
    pub async fn all_with_versions(
        conn: ConnectionPool,
    ) -> Result<HashMap<Crate, Vec<CrateVersion<'static>>>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let crate_versions = crates::table
                .inner_join(crate_versions::table)
                .load(&conn)?;

            Ok(crate_versions.into_iter().into_grouping_map().collect())
        })
        .await?
    }

    pub async fn all_visible_with_versions(
        conn: ConnectionPool,
        given_user_id: i32,
    ) -> Result<HashMap<Crate, Vec<CrateVersion<'static>>>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let crate_versions = crates::table
                .inner_join(crate::schema::user_crate_permissions::table)
                .filter(
                    crate::schema::user_crate_permissions::permissions
                        .bitwise_and(crate::users::UserCratePermissionValue::VISIBLE.bits())
                        .ne(0),
                )
                .filter(crate::schema::user_crate_permissions::dsl::user_id.eq(given_user_id))
                .inner_join(crate_versions::table)
                .select((crates::all_columns, crate_versions::all_columns))
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

    pub async fn versions(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Vec<CrateVersion<'static>>> {
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
    ) -> Result<Option<CrateVersion<'static>>> {
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
            use crate::schema::user_crate_permissions::dsl::permissions;

            let conn = conn.get()?;

            Ok(UserCratePermission::belonging_to(&*self)
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

    pub async fn members(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Vec<(crate::users::User, crate::users::UserCratePermissionValue)>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(UserCratePermission::belonging_to(&*self)
                .inner_join(crate::schema::users::dsl::users)
                .select((
                    crate::schema::users::all_columns,
                    crate::schema::user_crate_permissions::permissions,
                ))
                .load(&conn)?)
        })
        .await?
    }

    pub async fn update_permissions(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
        given_permissions: crate::users::UserCratePermissionValue,
    ) -> Result<usize> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::{
                crate_id, permissions, user_crate_permissions, user_id,
            };

            let conn = conn.get()?;

            Ok(diesel::update(
                user_crate_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(crate_id.eq(self.id)),
            )
            .set(permissions.eq(given_permissions.bits()))
            .execute(&conn)?)
        })
        .await?
    }

    pub async fn delete_member(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
    ) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::{
                crate_id, user_crate_permissions, user_id,
            };

            let conn = conn.get()?;

            diesel::delete(
                user_crate_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(crate_id.eq(self.id))
            )
            .execute(&conn)?;

            Ok(())
        })
        .await?
    }

    pub async fn publish_version(
        self: Arc<Self>,
        conn: ConnectionPool,
        file_identifier: chartered_fs::FileReference,
        file_checksum: String,
        given: chartered_types::cargo::CrateVersion<'static>,
        metadata: chartered_types::cargo::CrateVersionMetadata,
    ) -> Result<()> {
        use crate::schema::crate_versions::dsl::{
            checksum, crate_id, crate_versions, dependencies, description, documentation, features,
            filesystem_object, homepage, links, readme, repository, version,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            insert_into(crate_versions)
                .values((
                    crate_id.eq(self.id),
                    filesystem_object.eq(file_identifier.to_string()),
                    checksum.eq(file_checksum),
                    version.eq(given.vers),
                    dependencies.eq(CrateDependencies(given.deps)),
                    features.eq(CrateFeatures(given.features)),
                    links.eq(given.links),
                    description.eq(metadata.description),
                    readme.eq(metadata.readme),
                    repository.eq(metadata.repository),
                    homepage.eq(metadata.homepage),
                    documentation.eq(metadata.documentation),
                ))
                .execute(&conn)?;

            Ok(())
        })
        .await?
    }

    pub async fn yank_version(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_version: String,
        yank: bool,
    ) -> Result<()> {
        use crate::schema::crate_versions::dsl::{crate_id, crate_versions, version, yanked};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            diesel::update(
                crate_versions
                    .filter(crate_id.eq(self.id))
                    .filter(version.eq(given_version)),
            )
            .set(yanked.eq(yank))
            .execute(&conn)?;

            Ok(())
        })
        .await?
    }
}

impl<'a> From<Vec<chartered_types::cargo::CrateDependency<'a>>> for CrateDependencies<'a> {
    fn from(o: Vec<chartered_types::cargo::CrateDependency<'a>>) -> Self {
        Self(o)
    }
}

impl<'a> From<chartered_types::cargo::CrateFeatures> for CrateFeatures {
    fn from(o: chartered_types::cargo::CrateFeatures) -> Self {
        Self(o)
    }
}
