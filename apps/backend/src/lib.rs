mod auth;
mod communities;
mod dms;
mod error;
pub mod models;
mod threads;
mod validation;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{path::Path, str::FromStr, time::Duration};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub async fn connect(database_url: &str) -> std::result::Result<SqlitePool, sqlx::Error> {
    connect_options(SqliteConnectOptions::from_str(database_url)?).await
}

pub async fn connect_default(
    home: &Path,
) -> std::result::Result<SqlitePool, Box<dyn std::error::Error>> {
    let directory = home.join(".local/share/herdlink");
    std::fs::create_dir_all(&directory)?;
    Ok(
        connect_options(SqliteConnectOptions::new().filename(directory.join("herdlink.db")))
            .await?,
    )
}

async fn connect_options(
    options: SqliteConnectOptions,
) -> std::result::Result<SqlitePool, sqlx::Error> {
    let options = options
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    // One connection serializes writes and also keeps in-memory databases consistent.
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

pub async fn migrate(db: &SqlitePool) -> std::result::Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(db).await
}

pub fn app(db: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/demo", post(auth::demo))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/me", get(auth::me))
        .route(
            "/api/communities",
            get(communities::list).post(communities::create),
        )
        .route("/api/communities/{id}", get(communities::get))
        .route("/api/communities/{id}/open", post(communities::open))
        .route("/api/communities/{id}/join", post(communities::join))
        .route("/api/communities/{id}/channels", get(communities::channels))
        .route(
            "/api/channels/{id}/threads",
            get(threads::list).post(threads::create),
        )
        .route("/api/threads/{id}", get(threads::get))
        .route(
            "/api/threads/{id}/comments",
            get(threads::comments).post(threads::comment),
        )
        .route("/api/dms", get(dms::list).post(dms::create))
        .route("/api/dms/{id}/messages", get(dms::messages).post(dms::send))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { db })
}

async fn health(State(state): State<AppState>) -> error::Result<Json<Value>> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(json!({ "status": "ok" })))
}
