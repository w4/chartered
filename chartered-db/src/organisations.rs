use crate::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    Error,
};

use super::{
    schema::{organisations, user_organisation_permissions, users},
    uuid::SqlUuid,
    ConnectionPool, Result,
};

use diesel::{prelude::*, Associations, Identifiable, Queryable};

use std::sync::Arc;

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct Organisation {
    pub id: i32,
    pub uuid: SqlUuid,
    pub name: String,
}

impl Organisation {
    pub async fn find_by_name(
        conn: ConnectionPool,
        requesting_user_id: i32,
        given_name: String,
    ) -> Result<OrganisationWithPermissions> {
        use organisations::dsl::name as organisation_name;

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            let (permissions, organisation) = organisations::table
                .left_join(
                    user_organisation_permissions::table.on(user_organisation_permissions::user_id
                        .eq(requesting_user_id)
                        .and(
                            user_organisation_permissions::organisation_id
                                .eq(organisations::dsl::id),
                        )),
                )
                .filter(organisation_name.eq(given_name))
                .select((
                    user_organisation_permissions::dsl::permissions.nullable(),
                    organisations::all_columns,
                ))
                .get_result::<(Option<Permission>, _)>(&conn)
                .optional()?
                .ok_or(Error::MissingOrganisation)?;

            let permissions =
                permissions.ok_or(Error::MissingOrganisationPermission(Permission::VISIBLE))?;

            Ok(OrganisationWithPermissions {
                organisation,
                permissions,
            })
        })
        .await?
    }
}

pub struct OrganisationWithPermissions {
    organisation: Organisation,
    permissions: Permission,
}

impl OrganisationWithPermissions {
    #[must_use]
    pub fn permissions(&self) -> Permission {
        self.permissions
    }

    pub async fn crates(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<Crate>> {
        if !self.permissions.contains(Permission::VISIBLE) {
            return Err(Error::MissingOrganisationPermission(Permission::VISIBLE));
        }

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;
            Crate::belonging_to(&self.organisation)
                .load(&conn)
                .map_err(Into::into)
        })
        .await?
    }

    pub async fn members(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<(User, Permission)>> {
        if !self.permissions.contains(Permission::VISIBLE) {
            return Err(Error::MissingOrganisationPermission(Permission::VISIBLE));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_organisation_permissions::dsl::organisation_id;

            let conn = conn.get()?;
            user_organisation_permissions::table
                .filter(organisation_id.eq(self.organisation.id))
                .inner_join(users::table)
                .select((
                    users::all_columns,
                    user_organisation_permissions::columns::permissions,
                ))
                .load(&conn)
                .map_err(Into::into)
        })
        .await?
    }

    pub async fn update_permissions(
        self: Arc<Self>,
        conn: ConnectionPool,
        given_user_id: i32,
        given_permissions: crate::users::UserCratePermissionValue,
    ) -> Result<usize> {
        if !self.permissions.contains(Permission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(Permission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_organisation_permissions::dsl::{
                organisation_id, permissions, user_id, user_organisation_permissions,
            };

            let conn = conn.get()?;

            Ok(diesel::update(
                user_organisation_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(organisation_id.eq(self.organisation.id)),
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
        given_permissions: crate::users::UserCratePermissionValue,
    ) -> Result<usize> {
        if !self.permissions.contains(Permission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(Permission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_organisation_permissions::dsl::{
                organisation_id, permissions, user_id, user_organisation_permissions,
            };

            let conn = conn.get()?;

            Ok(diesel::insert_into(user_organisation_permissions)
                .values((
                    user_id.eq(given_user_id),
                    organisation_id.eq(self.organisation.id),
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
        if !self.permissions.contains(Permission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(Permission::MANAGE_USERS));
        }

        tokio::task::spawn_blocking(move || {
            use crate::schema::user_organisation_permissions::dsl::{
                organisation_id, user_id, user_organisation_permissions,
            };

            let conn = conn.get()?;

            diesel::delete(
                user_organisation_permissions
                    .filter(user_id.eq(given_user_id))
                    .filter(organisation_id.eq(self.organisation.id)),
            )
            .execute(&conn)?;

            Ok(())
        })
        .await?
    }
}
