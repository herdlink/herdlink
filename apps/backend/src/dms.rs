use crate::{
    AppState,
    auth::AuthUser,
    error::{AppError, Result},
    models::{DmConversation, DmMessage},
    validation::{self, Pagination},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

async fn require_participant(state: &AppState, conversation_id: Uuid, user_id: Uuid) -> Result<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM dm_conversations WHERE id = ?1 AND (user_low_id = ?2 OR user_high_id = ?2))")
        .bind(conversation_id).bind(user_id).fetch_one(&state.db).await?;
    if !exists {
        return Err(AppError::not_found());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateConversation {
    recipient_id: Uuid,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateConversation>,
) -> Result<Json<DmConversation>> {
    if auth.id == input.recipient_id {
        return Err(AppError::bad_request("cannot start a DM with yourself"));
    }
    let (low, high) = (
        auth.id.min(input.recipient_id),
        auth.id.max(input.recipient_id),
    );
    Ok(Json(sqlx::query_as("INSERT INTO dm_conversations (id, user_low_id, user_high_id) VALUES (?3, ?1, ?2) ON CONFLICT (user_low_id, user_high_id) DO UPDATE SET user_low_id = EXCLUDED.user_low_id RETURNING *")
        .bind(low).bind(high).bind(Uuid::new_v4()).fetch_one(&state.db).await?))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(page): Query<Pagination>,
) -> Result<Json<Vec<DmConversation>>> {
    let (limit, offset) = page.bounds()?;
    Ok(Json(sqlx::query_as("SELECT * FROM dm_conversations WHERE user_low_id = ?1 OR user_high_id = ?1 ORDER BY created_at DESC, id DESC LIMIT ?2 OFFSET ?3")
        .bind(auth.id).bind(limit).bind(offset).fetch_all(&state.db).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendMessage {
    body: String,
}

pub async fn send(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<SendMessage>,
) -> Result<(StatusCode, Json<DmMessage>)> {
    require_participant(&state, id, auth.id).await?;
    let body = validation::text(&input.body, 10000)?;
    Ok((StatusCode::CREATED, Json(sqlx::query_as("INSERT INTO dm_messages (id, conversation_id, author_id, body) VALUES (?4, ?1, ?2, ?3) RETURNING *")
        .bind(id).bind(auth.id).bind(body).bind(Uuid::new_v4()).fetch_one(&state.db).await?)))
}

pub async fn messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(page): Query<Pagination>,
) -> Result<Json<Vec<DmMessage>>> {
    require_participant(&state, id, auth.id).await?;
    let (limit, offset) = page.bounds()?;
    Ok(Json(sqlx::query_as("SELECT * FROM dm_messages WHERE conversation_id = ?1 ORDER BY created_at DESC, id DESC LIMIT ?2 OFFSET ?3")
        .bind(id).bind(limit).bind(offset).fetch_all(&state.db).await?))
}
