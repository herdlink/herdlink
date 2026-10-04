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

// Opening a community is an explicit, idempotent mutation; GET remains read-only.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenCommunity {
    name: Option<String>,
}

fn default_name(slug: &str) -> String {
    if Uuid::parse_str(slug).is_ok() {
        format!("Community {}", &slug[..8])
    } else {
        slug.split('-')
            .map(|word| format!("{}{}", word[..1].to_ascii_uppercase(), &word[1..]))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub async fn open(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(key): Path<String>,
    input: Option<Json<OpenCommunity>>,
) -> Result<Json<Community>> {
    let mut display_name = input
        .and_then(|Json(input)| input.name)
        .map(|name| validation::text(&name, 100))
        .transpose()?;
    let key = key.trim().to_ascii_lowercase();
    let requested_id = Uuid::parse_str(&key).ok();
    let slug = validation::slug(&if key.len() < 3 {
        format!("community-{key}")
    } else {
        key.clone()
    })?;
    // Direct visits (including existing directory links) also resolve disease titles.
    // Keep the network read outside the SQLite transaction and fall back if unavailable.
    if display_name.is_none() {
        let lookup_slug: Option<String> = if let Some(id) = requested_id {
            sqlx::query_scalar("SELECT slug FROM communities WHERE id = ?1")
                .bind(id)
                .fetch_optional(&state.db)
                .await?
        } else {
            Some(slug.clone())
        };
        if let Some(lookup_slug) = lookup_slug {
            display_name = state.graph.community_name(&lookup_slug).await;
        }
    }
    let id = requested_id.unwrap_or_else(Uuid::new_v4);
    let name = display_name.clone().unwrap_or_else(|| default_name(&slug));
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO communities (id, slug, name, created_by) SELECT ?1, ?2, ?3, ?4 WHERE NOT EXISTS (SELECT 1 FROM communities WHERE id = ?1 OR slug = ?2) ON CONFLICT DO NOTHING")
        .bind(id).bind(&slug).bind(name).bind(auth.id).execute(&mut *tx).await?;
    let mut community: Community = sqlx::query_as(
        "SELECT * FROM communities WHERE id = ?1 OR slug = ?2 ORDER BY (id = ?1) DESC LIMIT 1",
    )
    .bind(id)
    .bind(&slug)
    .fetch_one(&mut *tx)
    .await?;
    // Upgrade an identifier-derived title without overwriting a chosen community name.
    if let Some(name) = display_name
        && community.name == default_name(&community.slug)
    {
        sqlx::query("UPDATE communities SET name = ?1 WHERE id = ?2 AND name = ?3")
            .bind(&name)
            .bind(community.id)
            .bind(&community.name)
            .execute(&mut *tx)
            .await?;
        community.name = name;
    }
    sqlx::query("INSERT INTO community_members (community_id, user_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING")
        .bind(community.id).bind(auth.id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO channels (id, community_id, slug, name, kind, position) VALUES (?2, ?1, 'announcements', 'Announcements', 'announcement', 0), (?3, ?1, 'discussions', 'Discussions', 'discussion', 1) ON CONFLICT (community_id, slug) DO NOTHING")
        .bind(community.id).bind(Uuid::new_v4()).bind(Uuid::new_v4()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(community))
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
