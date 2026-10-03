use crate::{
    AppState,
    auth::AuthUser,
    error::{AppError, Result},
    models::{Channel, Community, Membership},
    validation::{self, Pagination},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn require_member(db: &SqlitePool, community_id: Uuid, user_id: Uuid) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM community_members WHERE community_id = ?1 AND user_id = ?2)",
    )
    .bind(community_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;
    if !exists {
        return Err(AppError::forbidden());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCommunity {
    slug: String,
    name: String,
    #[serde(default)]
    description: String,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateCommunity>,
) -> Result<(StatusCode, Json<Community>)> {
    let slug = validation::slug(&input.slug)?;
    let name = validation::text(&input.name, 100)?;
    let description = input.description.trim();
    if description.chars().count() > 5000 || description.contains('\0') {
        return Err(AppError::bad_request("invalid description"));
    }
    let mut tx = state.db.begin().await?;
    let community = sqlx::query_as::<_, Community>("INSERT INTO communities (id, slug, name, description, created_by) VALUES (?5, ?1, ?2, ?3, ?4) RETURNING *")
        .bind(slug).bind(name).bind(description).bind(auth.id).bind(Uuid::new_v4()).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO community_members (community_id, user_id) VALUES (?1, ?2)")
        .bind(community.id)
        .bind(auth.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO channels (id, community_id, slug, name, kind, position) VALUES (?2, ?1, 'announcements', 'Announcements', 'announcement', 0), (?3, ?1, 'discussions', 'Discussions', 'discussion', 1)")
        .bind(community.id).bind(Uuid::new_v4()).bind(Uuid::new_v4()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(community)))
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(page): Query<Pagination>,
) -> Result<Json<Vec<Community>>> {
    let (limit, offset) = page.bounds()?;
    Ok(Json(
        sqlx::query_as(
            "SELECT * FROM communities ORDER BY created_at DESC, id DESC LIMIT ?1 OFFSET ?2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
    ))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Community>> {
    Ok(Json(
        sqlx::query_as("SELECT * FROM communities WHERE id = ?1")
            .bind(id)
            .fetch_one(&state.db)
            .await?,
    ))
}

pub async fn join(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Membership>> {
    Ok(Json(sqlx::query_as("INSERT INTO community_members (community_id, user_id) VALUES (?1, ?2) ON CONFLICT (community_id, user_id) DO UPDATE SET user_id = EXCLUDED.user_id RETURNING *")
        .bind(id).bind(auth.id).fetch_one(&state.db).await?))
}

pub async fn channels(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Channel>>> {
    require_member(&state.db, id, auth.id).await?;
    Ok(Json(
        sqlx::query_as("SELECT * FROM channels WHERE community_id = ?1 ORDER BY position, id")
            .bind(id)
            .fetch_all(&state.db)
            .await?,
    ))
}
