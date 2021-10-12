use crate::{
    coalesce, crates::Crate, permissions::UserPermission, users::User, BitwiseExpressionMethods,
    Error,
};

use super::{
    schema::{organisations, user_organisation_permissions, users},
    uuid::SqlUuid,
    ConnectionPool, Result,
};

use diesel::{prelude::*, Associations, Identifiable, Queryable};

use std::sync::Arc;

macro_rules! select_permissions {
    () => {
        coalesce(
            crate::schema::user_organisation_permissions::permissions.nullable(),
            0,
        )
        .bitwise_or(diesel::dsl::sql::<diesel::sql_types::Integer>(&format!(
            "COALESCE(CASE WHEN {} THEN {} OR 0 END, 0)",
            "public",
            UserPermission::VISIBLE.bits(),
        )))
    };
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Eq, Hash, Debug)]
pub struct Organisation {
    pub id: i32,
    pub uuid: SqlUuid,
    pub name: String,
    pub description: String,
    pub public: bool,
}

impl Organisation {
    pub async fn list(conn: ConnectionPool, requesting_user_id: i32) -> Result<Vec<Organisation>> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            organisations::table
                .left_join(
                    user_organisation_permissions::table.on(user_organisation_permissions::user_id
                        .eq(requesting_user_id)
                        .and(
                            user_organisation_permissions::organisation_id
                                .eq(organisations::dsl::id),
                        )),
                )
                .filter(
                    select_permissions!()
                        .bitwise_and(UserPermission::VISIBLE.bits())
                        .eq(UserPermission::VISIBLE.bits()),
                )
                .select(organisations::all_columns)
                .load(&conn)
                .map_err(Into::into)
        })
        .await?
    }

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
                .select((select_permissions!(), organisations::all_columns))
                .get_result(&conn)
                .optional()?
                .ok_or(Error::MissingOrganisation)?;

            Ok(OrganisationWithPermissions {
                organisation,
                permissions,
            })
        })
        .await?
    }

    pub async fn create(
        conn: ConnectionPool,
        given_name: String,
        given_description: String,
        given_public: bool,
        requesting_user_id: i32,
    ) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;

            conn.transaction::<_, crate::Error, _>(|| {
                use organisations::dsl::{description, id, name, uuid, public};
                use user_organisation_permissions::dsl::{organisation_id, permissions, user_id};

                let generated_uuid = SqlUuid::random();

                diesel::insert_into(organisations::table)
                    .values((
                        uuid.eq(generated_uuid),
                        name.eq(given_name),
                        description.eq(given_description),
                        public.eq(given_public),
                    ))
                    .execute(&conn)?;

                let inserted_id: i32 = organisations::table
                    .filter(uuid.eq(generated_uuid))
                    .select(id)
                    .get_result(&conn)?;

                diesel::insert_into(user_organisation_permissions::table)
                    .values((
                        user_id.eq(requesting_user_id),
                        organisation_id.eq(inserted_id),
                        permissions.eq(UserPermission::all().bits()),
                    ))
                    .execute(&conn)?;

                Ok(())
            })?;

            Ok(())
        })
        .await?
    }
}

pub struct OrganisationWithPermissions {
    organisation: Organisation,
    permissions: UserPermission,
}

impl OrganisationWithPermissions {
    #[must_use]
    pub fn permissions(&self) -> UserPermission {
        self.permissions
    }

    #[must_use]
    pub fn organisation(&self) -> &Organisation {
        &self.organisation
    }

    pub async fn crates(self: Arc<Self>, conn: ConnectionPool) -> Result<Vec<Crate>> {
        if !self.permissions.contains(UserPermission::VISIBLE) {
            return Err(Error::MissingOrganisationPermission(
                UserPermission::VISIBLE,
            ));
        }

        tokio::task::spawn_blocking(move || {
            let conn = conn.get()?;
            Crate::belonging_to(&self.organisation)
                .load(&conn)
                .map_err(Into::into)
        })
        .await?
    }

    pub async fn members(
        self: Arc<Self>,
        conn: ConnectionPool,
    ) -> Result<Vec<(User, UserPermission)>> {
        if !self.permissions.contains(UserPermission::VISIBLE) {
            return Err(Error::MissingOrganisationPermission(
                UserPermission::VISIBLE,
            ));
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
        given_permissions: UserPermission,
    ) -> Result<usize> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
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
        given_permissions: UserPermission,
    ) -> Result<usize> {
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
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
        if !self.permissions.contains(UserPermission::MANAGE_USERS) {
            return Err(Error::MissingCratePermission(UserPermission::MANAGE_USERS));
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
