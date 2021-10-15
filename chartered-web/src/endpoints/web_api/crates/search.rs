use axum::{extract, Json};
use chartered_db::{crates::Crate, permissions::UserPermission, users::User, ConnectionPool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Deserialize)]
pub struct RequestParams {
    q: String,
}

#[derive(Serialize)]
pub struct Response {
    crates: Vec<ResponseCrate>,
}

#[derive(Serialize)]
pub struct ResponseCrate {
    organisation: String,
    name: String,
    description: Option<String>,
    version: String,
    homepage: Option<String>,
    repository: Option<String>,
    permissions: UserPermission,
}

pub async fn handle(
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Query(req): extract::Query<RequestParams>,
) -> Result<Json<Response>, Error> {
    let crates = futures::future::try_join_all(
        Crate::search(db.clone(), user.id, req.q, 5)
            .await?
            .into_iter()
            .flat_map(move |(org, crates_with_permissions)| {
                let db = db.clone();

                crates_with_permissions
                    .into_iter()
                    .map(Arc::new)
                    .map(move |v| {
                        let db = db.clone();
                        let org_name = org.name.clone();

                        async move {
                            let version = v.clone().latest_version(db).await?;

                            Ok::<_, Error>(ResponseCrate {
                                organisation: org_name,
                                name: v.crate_.name.clone(),
                                description: v.crate_.description.clone(),
                                version: version.map(|v| v.version).unwrap_or_default(),
                                homepage: v.crate_.homepage.clone(),
                                repository: v.crate_.repository.clone(),
                                permissions: v.permissions,
                            })
                        }
                    })
            }),
    )
    .await?;

    Ok(Json(Response { crates }))
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

define_error_response!(Error);
