use super::{
    coalesce,
    organisations::Organisation,
    permissions::UserPermission,
    schema::{crate_versions, crates, organisations, user_crate_permissions, users},
    users::User,
    BitwiseExpressionMethods, ConnectionPool, Error, Result,
};
use diesel::{insert_into, prelude::*, Associations, Identifiable, Queryable};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Identifiable, Queryable, Associations, Default, PartialEq, Eq, Hash, Debug)]
#[belongs_to(User)]
#[belongs_to(Crate)]
pub struct UserCratePermission {
    pub id: i32,
    pub user_id: i32,
    pub crate_id: i32,
    pub permissions: UserPermission,
}

impl UserCratePermission {
    pub async fn find(
        conn: ConnectionPool,
        given_user_id: i32,
        given_crate_id: i32,
    ) -> Result<Option<UserCratePermission>> {
        use crate::schema::user_crate_permissions::dsl::{crate_id, user_id};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(crate::schema::user_crate_permissions::table
                .filter(user_id.eq(given_user_id))
                .filter(crate_id.eq(given_crate_id))
                .get_result(&conn)
                .optional()?)
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
#[belongs_to(Organisation)]
pub struct Crate {
    pub id: i32,
    pub name: String,
    pub organisation_id: i32,
    pub readme: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
}

macro_rules! crate_with_permissions {
    ($user_id:ident) => {
        crates::table
            .left_join(
                crate::schema::user_crate_permissions::table.on(
                    crate::schema::user_crate_permissions::dsl::user_id
                        .eq($user_id)
                        .and(crate::schema::user_crate_permissions::crate_id.eq(crates::id)),
                ),
            )
            .left_join(
                crate::schema::user_organisation_permissions::table.on(
                    crate::schema::user_organisation_permissions::user_id
                        .eq($user_id)
                        .and(
                            crate::schema::user_organisation_permissions::organisation_id
                                .eq(crates::organisation_id),
                        ),
                ),
            )
    };
}

macro_rules! select_permissions {
    () => {
        coalesce(
            crate::schema::user_crate_permissions::permissions.nullable(),
            0,
        )
        .bitwise_or(coalesce(
            crate::schema::user_organisation_permissions::permissions.nullable(),
            0,
        ))
    };
}

impl Crate {
    pub async fn search(
        conn: ConnectionPool,
        requesting_user_id: i32,
        terms: String,
        limit: i64,
    ) -> Result<HashMap<Organisation, Vec<CrateWithPermissions>>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let crates = crate_with_permissions!(requesting_user_id)
                .inner_join(organisations::table)
                .filter(
                    select_permissions!()
                        .bitwise_and(UserPermission::VISIBLE.bits())
                        .eq(UserPermission::VISIBLE.bits()),
                )
                .filter(
                    (organisations::name.concat("/").concat(crates::name))
                        .like(&format!("%{}%", terms)),
                )
                .select((
                    organisations::all_columns,
                    crates::all_columns,
                    select_permissions!(),
                ))
                .limit(limit)
                .load(&conn)?
                .into_iter()
                .map(|(organisation, crate_, permissions)| {
                    (
                        organisation,
                        CrateWithPermissions {
                            crate_,
                            permissions,
                        },
                    )
                })
                .into_group_map();

            Ok(crates)
        })
        .await?
    }

    pub async fn list_with_versions(
        conn: ConnectionPool,
        requesting_user_id: i32,
        given_org_name: String,
    ) -> Result<HashMap<Crate, Vec<CrateVersion<'static>>>> {
        use crate::schema::organisations::dsl::{name as org_name, organisations};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let crate_versions = crate_with_permissions!(requesting_user_id)
                .inner_join(organisations)
                .filter(org_name.eq(given_org_name))
                .filter(
                    select_permissions!()
                        .bitwise_and(UserPermission::VISIBLE.bits())
                        .eq(UserPermission::VISIBLE.bits()),
                )
                .inner_join(crate_versions::table)
                .select((crates::all_columns, crate_versions::all_columns))
                .load(&conn)?;

            Ok(crate_versions.into_iter().into_grouping_map().collect())
        })
        .await?
    }

    pub async fn list_recently_updated(
        conn: ConnectionPool,
        requesting_user_id: i32,
    ) -> Result<Vec<(Crate, CrateVersion<'static>, Organisation)>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let crates = crate_with_permissions!(requesting_user_id)
                .filter(
                    select_permissions!()
                        .bitwise_and(UserPermission::VISIBLE.bits())
                        .eq(UserPermission::VISIBLE.bits()),
                )
                .inner_join(organisations::table)
                .inner_join(crate_versions::table)
                .select((
                    crates::all_columns,
                    crate_versions::all_columns,
                    organisations::all_columns,
                ))
                .limit(10)
                .order_by(crate::schema::crate_versions::dsl::id.desc())
                .load(&conn)?;

            Ok(crates)
        })
        .await?
    }

    pub async fn find_by_name(
        conn: ConnectionPool,
        requesting_user_id: i32,
        given_org_name: String,
        given_crate_name: String,
    ) -> Result<CrateWithPermissions> {
        use crate::schema::crates::dsl::name as crate_name;
        use crate::schema::organisations::dsl::{name as org_name, organisations};

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let (crate_, permissions) = crate_with_permissions!(requesting_user_id)
                .inner_join(organisations)
                .filter(org_name.eq(given_org_name))
                .filter(crate_name.eq(given_crate_name))
                .select((crate::schema::crates::all_columns, select_permissions!()))
                .first::<(Crate, UserPermission)>(&conn)
                .optional()?
                .ok_or(Error::MissingCrate)?;

            if permissions.contains(UserPermission::VISIBLE) {
                Ok(CrateWithPermissions {
                    crate_,
                    permissions,
                })
            } else {
                Err(Error::MissingCratePermission(UserPermission::VISIBLE))
            }
        })
        .await?
    }

    pub async fn create(
        conn: ConnectionPool,
        requesting_user_id: i32,
        given_org_name: String,
        given_crate_name: String,
    ) -> Result<CrateWithPermissions> {
        use crate::schema::organisations::dsl::{id, name as org_name, organisations};
        use crate::schema::user_organisation_permissions::dsl::{
            organisation_id, permissions, user_id,
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let (org_id, perms) = organisations
                .filter(org_name.eq(given_org_name))
                .inner_join(
                    crate::schema::user_organisation_permissions::table
                        .on(organisation_id.eq(id).and(user_id.eq(requesting_user_id))),
                )
                .select((id, permissions))
                .first::<(i32, UserPermission)>(&conn)?;

            #[allow(clippy::if_not_else)]
            if !perms.contains(UserPermission::VISIBLE) {
                Err(Error::MissingCratePermission(UserPermission::VISIBLE))
            } else if !perms.contains(UserPermission::CREATE_CRATE) {
                Err(Error::MissingCratePermission(UserPermission::CREATE_CRATE))
            } else {
                use crate::schema::crates::dsl::{crates, name, organisation_id};

                insert_into(crates)
                    .values((name.eq(&given_crate_name), organisation_id.eq(org_id)))
                    .execute(&conn)?;

                let crate_ = crates
                    .filter(name.eq(given_crate_name).and(organisation_id.eq(org_id)))
                    .select(crate::schema::crates::all_columns)
                    .first::<Crate>(&conn)?;

                Ok(CrateWithPermissions {
                    crate_,
                    permissions: perms,
                })
            }
        })
        .await?
    }
}

#[derive(Debug)]
pub struct CrateWithPermissions {
    pub crate_: Crate,
    pub permissions: UserPermission,
}

impl CrateWithPermissions {
    pub async fn latest_version(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Option<CrateVersion<'static>>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(CrateVersion::belonging_to(&self.crate_)
                .order_by(crate_versions::id.desc())
                .limit(1)
                .get_result::<CrateVersion>(&conn)
                .optional()?)
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

            Ok(CrateVersion::belonging_to(&self.crate_)
                .filter(version.eq(crate_version))
                .get_result::<CrateVersion>(&conn)
                .optional()?)
        })
        .await?
    }

    pub async fn versions_with_uploader(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Vec<(CrateVersion<'static>, User)>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(CrateVersion::belonging_to(&self.crate_)
                .inner_join(users::table)
                .load::<(CrateVersion, User)>(&conn)?)
        })
        .await?
    }

    pub async fn owners(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<crate::users::User>> {
        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::permissions;

            let conn = conn.get()?;

            Ok(UserCratePermission::belonging_to(&self.crate_)
                .filter(
                    permissions
                        .bitwise_and(UserPermission::MANAGE_USERS.bits())
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
    ) -> Result<Vec<(crate::users::User, UserPermission)>> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            Ok(UserCratePermission::belonging_to(&self.crate_)
                .inner_join(users::dsl::users)
                .select((users::all_columns, user_crate_permissions::permissions))
                .load(&conn)?)
        })
        .await?
    }

    pub async fn update_permissions(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
        given_permissions: UserPermission,
    ) -> Result<usize> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::{
                crate_id, permissions, user_crate_permissions, user_id,
            };

            let conn = conn.get()?;

            Ok(diesel::update(
                user_crate_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(crate_id.eq(self.crate_.id)),
            )
            .set(permissions.eq(given_permissions.bits()))
            .execute(&conn)?)
        })
        .await?
    }

    pub async fn insert_permissions(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
        given_permissions: UserPermission,
    ) -> Result<usize> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::{
                crate_id, permissions, user_crate_permissions, user_id,
            };

            let conn = conn.get()?;

            Ok(diesel::insert_into(user_crate_permissions)
                .values((
                    user_id.eq(given_user_id),
                    crate_id.eq(self.crate_.id),
                    permissions.eq(given_permissions.bits()),
                ))
                .execute(&conn)?)
        })
        .await?
    }

    pub async fn delete_member(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
    ) -> Result<()> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_crate_permissions::dsl::{
                crate_id, user_crate_permissions, user_id,
            };

            let conn = conn.get()?;

            diesel::delete(
                user_crate_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(crate_id.eq(self.crate_.id)),
            )
            .execute(&conn)?;

            Ok(())
        })
        .await?
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_version(
        self: Arc<Self>,
        conn: ConnectionPool,
        user: Arc<User>,
        file_identifier: chartered_fs::FileReference,
        file_checksum: String,
        file_size: i32,
        given: chartered_types::cargo::CrateVersion<'static>,
        metadata: chartered_types::cargo::CrateVersionMetadata,
    ) -> Result<()> {
        use crate::schema::{
            crate_versions::dsl::{
                checksum, crate_id, crate_versions, dependencies, features, filesystem_object,
                links, size, user_id, version,
            },
            crates::dsl::{
                crates, description, documentation, homepage, id, name, readme, repository,
            },
        };

        if !self.permissions.contains(UserPermission::PUBLISH_VERSION) {
            return Err(Error::MissingCratePermission(
                UserPermission::PUBLISH_VERSION,
            ));
        }

        tokio::task::spawn_blocking(move || {
            use diesel::result::{DatabaseErrorKind, Error as DieselError};

            let conn = conn.get()?;

            conn.transaction::<_, crate::Error, _>(|| {
                diesel::update(crates.filter(id.eq(self.crate_.id)))
                    .set((
                        name.eq(given.name),
                        description.eq(metadata.description),
                        readme.eq(metadata.readme),
                        repository.eq(metadata.repository),
                        homepage.eq(metadata.homepage),
                        documentation.eq(metadata.documentation),
                    ))
                    .execute(&conn)?;

                let res = insert_into(crate_versions)
                    .values((
                        crate_id.eq(self.crate_.id),
                        filesystem_object.eq(file_identifier.to_string()),
                        size.eq(file_size),
                        checksum.eq(file_checksum),
                        version.eq(&given.vers),
                        dependencies.eq(CrateDependencies(given.deps)),
                        features.eq(CrateFeatures(given.features)),
                        links.eq(given.links),
                        user_id.eq(user.id),
                    ))
                    .execute(&conn);

                match res {
                    Ok(_) => Ok(()),
                    Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                        Err(Error::VersionConflict(given.vers.into_owned()))
                    }
                    Err(e) => Err(e.into()),
                }
            })?;

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

        if !self.permissions.contains(UserPermission::YANK_VERSION) {
            return Err(Error::MissingCratePermission(UserPermission::YANK_VERSION));
        }

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            diesel::update(
                crate_versions
                    .filter(crate_id.eq(self.crate_.id))
                    .filter(version.eq(given_version)),
            )
            .set(yanked.eq(yank))
            .execute(&conn)?;

            Ok(())
        })
        .await?
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Crate)]
#[belongs_to(User)]
pub struct CrateVersion<'a> {
    pub id: i32,
    pub crate_id: i32,
    pub version: String,
    pub filesystem_object: String,
    pub size: i32,
    pub yanked: bool,
    pub checksum: String,
    pub dependencies: CrateDependencies<'a>,
    pub features: CrateFeatures,
    pub links: Option<String>,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

impl<'a> CrateVersion<'a> {
    #[must_use]
    pub fn into_cargo_format(self, crate_: &'a Crate) -> chartered_types::cargo::CrateVersion<'a> {
        chartered_types::cargo::CrateVersion {
            name: crate_.name.as_str().into(),
            vers: self.version.into(),
            deps: self.dependencies.0,
            features: self.features.0,
            links: self.links.map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Debug, Clone, PartialEq, Eq)]
#[sql_type = "diesel::sql_types::Blob"]
pub struct CrateDependencies<'a>(pub Vec<chartered_types::cargo::CrateDependency<'a>>);

derive_diesel_json!(CrateDependencies<'a>);

impl<'a> From<Vec<chartered_types::cargo::CrateDependency<'a>>> for CrateDependencies<'a> {
    fn from(o: Vec<chartered_types::cargo::CrateDependency<'a>>) -> Self {
        Self(o)
    }
}

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Debug, Clone, PartialEq, Eq)]
#[sql_type = "diesel::sql_types::Blob"]
pub struct CrateFeatures(pub chartered_types::cargo::CrateFeatures);

derive_diesel_json!(CrateFeatures);

impl<'a> From<chartered_types::cargo::CrateFeatures> for CrateFeatures {
    fn from(o: chartered_types::cargo::CrateFeatures) -> Self {
        Self(o)
    }
}
