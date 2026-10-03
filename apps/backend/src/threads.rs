use crate::{
    AppState,
    auth::AuthUser,
    communities::require_member,
    error::Result,
    models::{Channel, Comment, Thread},
    validation::{self, Pagination},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateThread {
    title: String,
    body: String,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(input): Json<CreateThread>,
) -> Result<(StatusCode, Json<Thread>)> {
    let title = validation::text(&input.title, 300)?;
    let body = validation::text(&input.body, 40000)?;
    let channel: Channel = sqlx::query_as("SELECT * FROM channels WHERE id = ?1")
        .bind(channel_id)
        .fetch_one(&state.db)
        .await?;
    require_member(&state.db, channel.community_id, auth.id).await?;
    Ok((StatusCode::CREATED, Json(sqlx::query_as("INSERT INTO threads (id, community_id, channel_id, author_id, title, body) VALUES (?6, ?1, ?2, ?3, ?4, ?5) RETURNING *")
        .bind(channel.community_id).bind(channel_id).bind(auth.id).bind(title).bind(body).bind(Uuid::new_v4()).fetch_one(&state.db).await?)))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(page): Query<Pagination>,
) -> Result<Json<Vec<Thread>>> {
    let (limit, offset) = page.bounds()?;
    let community_id: Uuid = sqlx::query_scalar("SELECT community_id FROM channels WHERE id = ?1")
        .bind(channel_id)
        .fetch_one(&state.db)
        .await?;
    require_member(&state.db, community_id, auth.id).await?;
    Ok(Json(sqlx::query_as("SELECT * FROM threads WHERE channel_id = ?1 ORDER BY created_at DESC, id DESC LIMIT ?2 OFFSET ?3")
        .bind(channel_id).bind(limit).bind(offset).fetch_all(&state.db).await?))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Thread>> {
    let thread: Thread = sqlx::query_as("SELECT * FROM threads WHERE id = ?1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    require_member(&state.db, thread.community_id, auth.id).await?;
    Ok(Json(thread))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateComment {
    body: String,
    parent_id: Option<Uuid>,
}

pub async fn comment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(thread_id): Path<Uuid>,
    Json(input): Json<CreateComment>,
) -> Result<(StatusCode, Json<Comment>)> {
    let body = validation::text(&input.body, 10000)?;
    let community_id: Uuid = sqlx::query_scalar("SELECT community_id FROM threads WHERE id = ?1")
        .bind(thread_id)
        .fetch_one(&state.db)
        .await?;
    require_member(&state.db, community_id, auth.id).await?;
    Ok((StatusCode::CREATED, Json(sqlx::query_as("INSERT INTO comments (id, community_id, thread_id, author_id, parent_id, body) VALUES (?6, ?1, ?2, ?3, ?4, ?5) RETURNING *")
        .bind(community_id).bind(thread_id).bind(auth.id).bind(input.parent_id).bind(body).bind(Uuid::new_v4()).fetch_one(&state.db).await?)))
}

pub async fn comments(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(thread_id): Path<Uuid>,
    Query(page): Query<Pagination>,
) -> Result<Json<Vec<Comment>>> {
    let (limit, offset) = page.bounds()?;
    let community_id: Uuid = sqlx::query_scalar("SELECT community_id FROM threads WHERE id = ?1")
        .bind(thread_id)
        .fetch_one(&state.db)
        .await?;
    require_member(&state.db, community_id, auth.id).await?;
    Ok(Json(sqlx::query_as("SELECT * FROM comments WHERE thread_id = ?1 ORDER BY created_at, id LIMIT ?2 OFFSET ?3")
        .bind(thread_id).bind(limit).bind(offset).fetch_all(&state.db).await?))
}
