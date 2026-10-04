use crate::{
    AppState,
    error::{AppError, Result},
    models::{User, UserRole},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct AuthUser {
    pub id: Uuid,
    token_hash: Vec<u8>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let token = parts
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|token| token.len() == 64 && token.bytes().all(|c| c.is_ascii_hexdigit()))
            .ok_or_else(AppError::unauthorized)?;
        let token_hash = Sha256::digest(token.as_bytes()).to_vec();
        let id = sqlx::query_scalar(
            "SELECT user_id FROM sessions WHERE token_hash = ?1 AND julianday(expires_at) > julianday('now')",
        )
        .bind(&token_hash)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(AppError::unauthorized)?;
        Ok(Self { id, token_hash })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    email: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

fn email(value: &str) -> Result<String> {
    let value = value.trim().to_ascii_lowercase();
    let valid_parts = value.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty() && !domain.is_empty() && !domain.contains('@') && domain.contains('.')
    });
    if value.len() > 254
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
        || !valid_parts
    {
        return Err(AppError::bad_request("invalid email address"));
    }
    Ok(value)
}

async fn hash_password(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(AppError::internal)
    })
    .await
    .map_err(AppError::internal)?
}

async fn new_session(user: User, db: &mut sqlx::SqliteConnection) -> Result<AuthResponse> {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = Sha256::digest(token.as_bytes()).to_vec();
    let expires_at = sqlx::query_scalar(
        "INSERT INTO sessions (token_hash, user_id) VALUES (?1, ?2) RETURNING expires_at",
    )
    .bind(hash)
    .bind(user.id)
    .fetch_one(db)
    .await?;
    Ok(AuthResponse {
        user,
        token,
        expires_at,
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<Register>,
) -> Result<(StatusCode, Json<AuthResponse>)> {
    let email = email(&input.email)?;
    let username = input.username.trim().to_ascii_lowercase();
    if !(3..=32).contains(&username.len())
        || !username
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err(AppError::bad_request(
            "username must be 3-32 letters, digits, or underscores",
        ));
    }
    if !(12..=1024).contains(&input.password.len()) {
        return Err(AppError::bad_request("password must be 12-1024 bytes"));
    }
    let hash = hash_password(input.password).await?;
    let mut tx = state.db.begin().await?;
    // New accounts always have the user role. Professional roles require trusted provisioning.
    let user = sqlx::query_as::<_, User>("INSERT INTO users (id, email, username, password_hash) VALUES (?4, ?1, ?2, ?3) RETURNING id, email, username, role, created_at")
        .bind(email).bind(username).bind(hash).bind(Uuid::new_v4()).fetch_one(&mut *tx).await?;
    let response = new_session(user, &mut tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<Login>,
) -> Result<Json<AuthResponse>> {
    let email = email(&input.email)?;
    if input.password.len() > 1024 {
        return Err(AppError::unauthorized());
    }
    let credentials: Option<(Uuid, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE email = ?1")
            .bind(email)
            .fetch_optional(&state.db)
            .await?;
    let Some((id, stored_hash)) = credentials else {
        // Perform the same expensive password operation for unknown accounts.
        hash_password(input.password).await?;
        return Err(AppError::unauthorized());
    };
    let verified = tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&stored_hash).map_err(AppError::internal)?;
        Ok::<_, AppError>(
            Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok(),
        )
    })
    .await
    .map_err(AppError::internal)??;
    if !verified {
        return Err(AppError::unauthorized());
    }
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, username, role, created_at FROM users WHERE id = ?1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    let mut connection = state.db.acquire().await?;
    Ok(Json(new_session(user, &mut connection).await?))
}

pub async fn me(State(state): State<AppState>, auth: AuthUser) -> Result<Json<User>> {
    Ok(Json(
        sqlx::query_as("SELECT id, email, username, role, created_at FROM users WHERE id = ?1")
            .bind(auth.id)
            .fetch_one(&state.db)
            .await?,
    ))
}

// Temporary shared demo identity. It still uses ordinary sessions and membership checks.
pub async fn demo(State(state): State<AppState>) -> Result<Json<AuthResponse>> {
    let id = Uuid::from_u128(0x378ba963_dcdc_4bce_84ce_6c3f45309e2b);
    let mut tx = state.db.begin().await?;
    let existing: Option<User> =
        sqlx::query_as("SELECT id, email, username, role, created_at FROM users WHERE id = ?1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let user = if let Some(user) = existing {
        user
    } else {
        let hash = hash_password(Uuid::new_v4().to_string()).await?;
        sqlx::query_as::<_, User>("INSERT INTO users (id, email, username, password_hash, role) VALUES (?1, 'demo@herdlink.local', 'demo_user', ?2, 'user') RETURNING id, email, username, role, created_at")
            .bind(id).bind(hash).fetch_one(&mut *tx).await?
    };
    if user.role != UserRole::User {
        return Err(AppError::forbidden());
    }
    let response = new_session(user, &mut tx).await?;
    tx.commit().await?;
    Ok(Json(response))
}

pub async fn logout(State(state): State<AppState>, auth: AuthUser) -> Result<StatusCode> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?1")
        .bind(auth.token_hash)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
