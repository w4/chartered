//! Password-based authentication, including registration and login.

use crate::config::Config;

use axum::{extract, Json};
use chartered_db::{users::User, ConnectionPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{net::IpAddr, sync::Arc};

pub async fn handle_register(
    extract::Extension(config): extract::Extension<Arc<Config>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Json(req): extract::Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, RegisterError> {
    // some basic validation before we register the user
    if !config.auth.password.enabled {
        return Err(RegisterError::PasswordAuthDisabled);
    } else if !validate_username(&req.username) {
        return Err(RegisterError::InvalidUsername);
    } else if req.password.len() < 6 {
        return Err(RegisterError::PasswordRequirementNotMet);
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;

    match User::register(db, req.username, password_hash).await {
        Ok(_) => Ok(Json(RegisterResponse { success: true })),
        Err(chartered_db::Error::UsernameTaken) => Err(RegisterError::UsernameTaken),
        Err(e) => Err(e.into()),
    }
}

pub async fn handle_login(
    extract::Extension(config): extract::Extension<Arc<Config>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Json(req): extract::Json<LoginRequest>,
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    addr: extract::Extension<IpAddr>,
) -> Result<Json<super::LoginResponse>, LoginError> {
    // some basic validation before we attempt a login
    if !config.auth.password.enabled {
        return Err(LoginError::PasswordAuthDisabled);
    } else if !validate_username(&req.username) {
        return Err(LoginError::UnknownUser);
    } else if req.password.is_empty() {
        return Err(LoginError::InvalidPassword);
    }

    let user = User::find_by_username(db.clone(), req.username)
        .await?
        .ok_or(LoginError::UnknownUser)?;

    let password_hash = user
        .password
        .as_deref()
        // password is nullable for openid logins
        .ok_or(LoginError::UnknownUser)?;

    if bcrypt::verify(&req.password, password_hash)? {
        Ok(Json(super::login(db, user, user_agent, addr).await?))
    } else {
        Err(LoginError::InvalidPassword)
    }
}

pub fn validate_username(username: &str) -> bool {
    // we use `:` as a splitter for openid logins so it isn't legal during password login
    !username.contains(':')
        // must have at least 1 character in the username
        && !username.is_empty()
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    success: bool,
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to hash password")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Username is invalid")]
    InvalidUsername,
    #[error("Username already taken")]
    UsernameTaken,
    #[error("Password authentication is disabled")]
    PasswordAuthDisabled,
    #[error("Password must be at least 6 characters long")]
    PasswordRequirementNotMet,
}

impl RegisterError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) | Self::Bcrypt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidUsername | Self::UsernameTaken | Self::PasswordRequirementNotMet => {
                StatusCode::BAD_REQUEST
            }
            Self::PasswordAuthDisabled => StatusCode::FORBIDDEN,
        }
    }
}

define_error_response!(RegisterError);

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to hash password")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Invalid username/password")]
    UnknownUser,
    #[error("Invalid username/password")]
    InvalidPassword,
    #[error("Password authentication is disabled")]
    PasswordAuthDisabled,
}

impl LoginError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) | Self::Bcrypt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownUser | Self::InvalidPassword | Self::PasswordAuthDisabled => {
                StatusCode::FORBIDDEN
            }
        }
    }
}

define_error_response!(LoginError);
