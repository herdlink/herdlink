use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn sqlite_file_persists_and_enforces_constraints() {
    let directory = tempfile::Builder::new()
        .prefix("herdlink test ")
        .tempdir()
        .unwrap();
    let path = directory.path().join(".local/share/herdlink/herdlink.db");
    assert!(!path.exists());
    let pool = backend::connect_default(directory.path()).await.unwrap();
    assert!(path.is_file());
    backend::migrate(&pool).await.unwrap();
    // Startup migrations are safe to rerun against an existing database.
    backend::migrate(&pool).await.unwrap();
    let foreign_keys: i32 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(foreign_keys, 1);
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(journal, "wal");
    let app = backend::app(pool.clone());
    let (_, token) = register(&app, "persistent_user").await;
    let error = sqlx::query("UPDATE users SET role = 'administrator'")
        .execute(&pool)
        .await
        .unwrap_err();
    assert!(error.as_database_error().unwrap().is_check_violation());
    pool.close().await;
    let reopened = backend::connect_default(directory.path()).await.unwrap();
    let app = backend::app(reopened.clone());
    let me = expect(
        &app,
        "GET",
        "/api/me",
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(me["username"], "persistent_user");
    assert_eq!(me["role"], "user");
    reopened.close().await;
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    (status, body)
}

async fn expect(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
    expected: StatusCode,
) -> Value {
    let (status, value) = request(app, method, path, token, body).await;
    assert_eq!(status, expected, "{method} {path}: {value}");
    value
}

async fn register(app: &Router, name: &str) -> (String, String) {
    let response = expect(app, "POST", "/api/auth/register", None,
        json!({"email": format!("{name}@example.com"), "username": name, "password": "a long test password"}), StatusCode::CREATED).await;
    (
        response["user"]["id"].as_str().unwrap().to_owned(),
        response["token"].as_str().unwrap().to_owned(),
    )
}

async fn community(app: &Router, token: &str, slug: &str) -> (String, String, String) {
    let created = expect(
        app,
        "POST",
        "/api/communities",
        Some(token),
        json!({"slug": slug, "name": "Wilson's disease"}),
        StatusCode::CREATED,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_owned();
    let channels = expect(
        app,
        "GET",
        &format!("/api/communities/{id}/channels"),
        Some(token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    let channels = channels.as_array().unwrap();
    assert_eq!(channels.len(), 2);
    let announcement = channels
        .iter()
        .find(|c| c["kind"] == "announcement")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let discussion = channels.iter().find(|c| c["kind"] == "discussion").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    (id, announcement, discussion)
}

#[sqlx::test(migrations = "./migrations")]
async fn authentication_roles_and_sessions(pool: SqlitePool) {
    let app = backend::app(pool.clone());
    expect(&app, "GET", "/health", None, json!(null), StatusCode::OK).await;
    expect(
        &app,
        "GET",
        "/api/me",
        None,
        json!(null),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    expect(&app, "POST", "/api/auth/register", None, json!({"email": "role@example.com", "username": "role", "password": "a long test password", "role": "scientist"}), StatusCode::UNPROCESSABLE_ENTITY).await;
    let (id, token) = register(&app, "alice").await;
    let me = expect(
        &app,
        "GET",
        "/api/me",
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(me["role"], "user");
    assert!(me.get("password_hash").is_none());
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?1")
        .bind(Uuid::parse_str(&id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert_ne!(hash, "a long test password");
    expect(&app, "POST", "/api/auth/register", None, json!({"email": "ALICE@example.com", "username": "another_alice", "password": "a long test password"}), StatusCode::CONFLICT).await;
    expect(
        &app,
        "POST",
        "/api/auth/login",
        None,
        json!({"email": "alice@example.com", "password": "wrong password"}),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    expect(
        &app,
        "POST",
        "/api/auth/login",
        None,
        json!({"email": "missing@example.com", "password": "wrong password"}),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let login = expect(
        &app,
        "POST",
        "/api/auth/login",
        None,
        json!({"email": "ALICE@example.com", "password": "a long test password"}),
        StatusCode::OK,
    )
    .await;
    let new_token = login["token"].as_str().unwrap();
    expect(
        &app,
        "POST",
        "/api/auth/logout",
        Some(new_token),
        json!(null),
        StatusCode::NO_CONTENT,
    )
    .await;
    expect(
        &app,
        "GET",
        "/api/me",
        Some(new_token),
        json!(null),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    expect(
        &app,
        "GET",
        "/api/me",
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    for role in ["scientist", "pharma_scout"] {
        sqlx::query("UPDATE users SET role = ?1 WHERE id = ?2")
            .bind(role)
            .bind(Uuid::parse_str(&id).unwrap())
            .execute(&pool)
            .await
            .unwrap();
        let me = expect(
            &app,
            "GET",
            "/api/me",
            Some(&token),
            json!(null),
            StatusCode::OK,
        )
        .await;
        assert_eq!(me["role"], role);
    }
    sqlx::query(
        "UPDATE sessions SET expires_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-1 second') WHERE token_hash = ?1",
    )
    .bind(Sha256::digest(token.as_bytes()).to_vec())
    .execute(&pool)
    .await
    .unwrap();
    expect(
        &app,
        "GET",
        "/api/me",
        Some(&token),
        json!(null),
        StatusCode::UNAUTHORIZED,
    )
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn communities_have_only_one_announcement_channel(pool: SqlitePool) {
    let app = backend::app(pool.clone());
    let (_, token) = register(&app, "creator").await;
    let (id, _, _) = community(&app, &token, "wilsons-disease").await;
    let error = sqlx::query("INSERT INTO channels (id, community_id, slug, name, kind) VALUES (?1, ?2, 'another-announcement', 'Another announcement', 'announcement')")
        .bind(Uuid::new_v4())
        .bind(Uuid::parse_str(&id).unwrap())
        .execute(&pool).await.unwrap_err();
    assert!(error.as_database_error().unwrap().is_unique_violation());
    // Each other community still gets its own announcement channel.
    let (other_id, _, _) = community(&app, &token, "another-community").await;
    for id in [id, other_id] {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM channels WHERE community_id = ?1 AND kind = 'announcement'",
        )
        .bind(Uuid::parse_str(&id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn communities_announcements_and_scoped_comments(pool: SqlitePool) {
    let app = backend::app(pool.clone());
    let (_, creator) = register(&app, "creator").await;
    let (member_id, member) = register(&app, "member").await;
    let (_, outsider) = register(&app, "outsider").await;
    let (community_id, announcements, discussions) =
        community(&app, &creator, "wilsons-disease").await;
    let join = format!("/api/communities/{community_id}/join");
    let membership = expect(
        &app,
        "POST",
        &join,
        Some(&creator),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert!(membership.get("role").is_none());
    assert_eq!(membership["community_id"], community_id);
    expect(
        &app,
        "POST",
        &join,
        Some(&member),
        json!(null),
        StatusCode::OK,
    )
    .await;
    expect(
        &app,
        "GET",
        &format!("/api/communities/{community_id}/channels"),
        Some(&outsider),
        json!(null),
        StatusCode::FORBIDDEN,
    )
    .await;
    let announce_path = format!("/api/channels/{announcements}/threads");
    let discussion_path = format!("/api/channels/{discussions}/threads");
    let post = json!({"title": "My experience", "body": "Here is my story."});
    expect(
        &app,
        "POST",
        &discussion_path,
        Some(&outsider),
        post.clone(),
        StatusCode::FORBIDDEN,
    )
    .await;
    expect(
        &app,
        "POST",
        &announce_path,
        Some(&member),
        post.clone(),
        StatusCode::CREATED,
    )
    .await;
    let announcement = expect(
        &app,
        "POST",
        &announce_path,
        Some(&creator),
        post.clone(),
        StatusCode::CREATED,
    )
    .await;
    expect(
        &app,
        "POST",
        &format!(
            "/api/threads/{}/comments",
            announcement["id"].as_str().unwrap()
        ),
        Some(&member),
        json!({"body": "Thanks for the update!"}),
        StatusCode::CREATED,
    )
    .await;
    expect(
        &app,
        "PATCH",
        &format!("/api/communities/{community_id}/members/{member_id}"),
        Some(&creator),
        json!({"role": "moderator"}),
        StatusCode::NOT_FOUND,
    )
    .await;
    let thread = expect(
        &app,
        "POST",
        &discussion_path,
        Some(&member),
        post.clone(),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(thread["community_id"], community_id);
    assert_eq!(thread["author_id"], member_id);
    let thread_id = thread["id"].as_str().unwrap();
    let comments_path = format!("/api/threads/{thread_id}/comments");
    expect(
        &app,
        "GET",
        &format!("/api/threads/{thread_id}"),
        Some(&outsider),
        json!(null),
        StatusCode::FORBIDDEN,
    )
    .await;
    expect(
        &app,
        "GET",
        &comments_path,
        Some(&outsider),
        json!(null),
        StatusCode::FORBIDDEN,
    )
    .await;
    expect(
        &app,
        "POST",
        &comments_path,
        Some(&outsider),
        json!({"body": "Unauthorized"}),
        StatusCode::FORBIDDEN,
    )
    .await;
    let comment = expect(
        &app,
        "POST",
        &comments_path,
        Some(&creator),
        json!({"body": "Thank you for sharing."}),
        StatusCode::CREATED,
    )
    .await;
    let reply = expect(
        &app,
        "POST",
        &comments_path,
        Some(&member),
        json!({"body": "You're welcome!", "parent_id": comment["id"]}),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(reply["parent_id"], comment["id"]);
    let comments = expect(
        &app,
        "GET",
        &comments_path,
        Some(&member),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(comments.as_array().unwrap().len(), 2);
    let other_thread = expect(
        &app,
        "POST",
        &discussion_path,
        Some(&member),
        post.clone(),
        StatusCode::CREATED,
    )
    .await;
    expect(
        &app,
        "POST",
        &format!(
            "/api/threads/{}/comments",
            other_thread["id"].as_str().unwrap()
        ),
        Some(&member),
        json!({"body": "Invalid parent", "parent_id": comment["id"]}),
        StatusCode::BAD_REQUEST,
    )
    .await;
    expect(
        &app,
        "POST",
        &discussion_path,
        Some(&member),
        json!({"title": " ", "body": "text"}),
        StatusCode::BAD_REQUEST,
    )
    .await;
    expect(
        &app,
        "GET",
        &format!("{discussion_path}?limit=0"),
        Some(&member),
        json!(null),
        StatusCode::BAD_REQUEST,
    )
    .await;
    let page = expect(
        &app,
        "GET",
        &format!("{discussion_path}?limit=1"),
        Some(&member),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(page.as_array().unwrap().len(), 1);
    let (_, _, other_channel) = community(&app, &creator, "another-community").await;
    // Direct database writes also cannot attach a thread to a different community's channel.
    let result = sqlx::query("INSERT INTO threads (id, community_id, channel_id, author_id, title, body) VALUES (?4, ?1, ?2, ?3, 'Invalid', 'Wrong community')")
        .bind(Uuid::parse_str(&community_id).unwrap()).bind(Uuid::parse_str(&other_channel).unwrap()).bind(Uuid::parse_str(&member_id).unwrap()).bind(Uuid::new_v4()).execute(&pool).await;
    let error = result.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("787")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn direct_messages_are_private_and_pairs_are_unique(pool: SqlitePool) {
    let app = backend::app(pool);
    let (alice_id, alice) = register(&app, "alice").await;
    let (bob_id, bob) = register(&app, "bob").await;
    let (_, outsider) = register(&app, "outsider").await;
    expect(
        &app,
        "POST",
        "/api/dms",
        Some(&alice),
        json!({"recipient_id": alice_id}),
        StatusCode::BAD_REQUEST,
    )
    .await;
    let dm = expect(
        &app,
        "POST",
        "/api/dms",
        Some(&alice),
        json!({"recipient_id": bob_id}),
        StatusCode::OK,
    )
    .await;
    let same_dm = expect(
        &app,
        "POST",
        "/api/dms",
        Some(&bob),
        json!({"recipient_id": alice_id}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(dm["id"], same_dm["id"]);
    let path = format!("/api/dms/{}/messages", dm["id"].as_str().unwrap());
    let message = expect(
        &app,
        "POST",
        &path,
        Some(&alice),
        json!({"body": "Private hello"}),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(message["author_id"], alice_id);
    let messages = expect(&app, "GET", &path, Some(&bob), json!(null), StatusCode::OK).await;
    assert_eq!(messages[0]["body"], "Private hello");
    expect(
        &app,
        "POST",
        &path,
        Some(&bob),
        json!({"body": "Hello back"}),
        StatusCode::CREATED,
    )
    .await;
    expect(
        &app,
        "GET",
        &path,
        Some(&outsider),
        json!(null),
        StatusCode::NOT_FOUND,
    )
    .await;
    expect(
        &app,
        "POST",
        &path,
        Some(&outsider),
        json!({"body": "Intrusion"}),
        StatusCode::NOT_FOUND,
    )
    .await;
    expect(
        &app,
        "POST",
        &path,
        Some(&alice),
        json!({"body": "Spoof", "author_id": bob_id}),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    let dms = expect(
        &app,
        "GET",
        "/api/dms",
        Some(&outsider),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert!(dms.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn community_open_persists_readable_names_and_upgrades_identifier_titles() {
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app(pool);
    let (_, token) = register(&app, "names").await;
    let path = "/api/communities/mondo-test/open";
    let old = expect(&app, "POST", path, Some(&token), json!({}), StatusCode::OK).await;
    assert_eq!(old["name"], "Mondo Test");
    let id = old["id"].as_str().unwrap();
    let channels_path = format!("/api/communities/{id}/channels");
    let channels = expect(
        &app,
        "GET",
        &channels_path,
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    let posts_path = format!(
        "/api/channels/{}/threads",
        channels[1]["id"].as_str().unwrap()
    );
    let post = expect(
        &app,
        "POST",
        &posts_path,
        Some(&token),
        json!({"title":"Welcome", "body":"Keep this post"}),
        StatusCode::CREATED,
    )
    .await;
    let renamed = expect(
        &app,
        "POST",
        path,
        Some(&token),
        json!({"name":"Huntington disease"}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(renamed["id"], old["id"]);
    assert_eq!(renamed["slug"], old["slug"]);
    assert_eq!(renamed["name"], "Huntington disease");
    let by_id = expect(
        &app,
        "POST",
        &format!("/api/communities/{id}/open"),
        Some(&token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(by_id["name"], renamed["name"]);
    let list = expect(
        &app,
        "GET",
        "/api/communities",
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(list[0]["name"], renamed["name"]);
    let unchanged_channels = expect(
        &app,
        "GET",
        &channels_path,
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(channels, unchanged_channels);
    let posts = expect(
        &app,
        "GET",
        &posts_path,
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(posts[0]["id"], post["id"]);
    let kept = expect(
        &app,
        "POST",
        path,
        Some(&token),
        json!({"name":"Different title"}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(kept["name"], renamed["name"]);
    let fresh = expect(
        &app,
        "POST",
        "/api/communities/mondo-other/open",
        Some(&token),
        json!({"name":"Parkinson disease"}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(fresh["name"], "Parkinson disease");
    let custom = expect(
        &app,
        "POST",
        "/api/communities",
        Some(&token),
        json!({"slug":"personal-room", "name":"Our conversations"}),
        StatusCode::CREATED,
    )
    .await;
    let kept = expect(
        &app,
        "POST",
        "/api/communities/personal-room/open",
        Some(&token),
        json!({"name":"Different title"}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(kept["name"], custom["name"]);
    for name in [String::new(), "a".repeat(101), "invalid\0name".into()] {
        expect(
            &app,
            "POST",
            path,
            Some(&token),
            json!({"name":name}),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
}

#[tokio::test]
async fn demo_community_open_is_idempotent_and_messages_persist() {
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app(pool.clone());
    let (first, second) = tokio::join!(
        expect(
            &app,
            "POST",
            "/api/auth/demo",
            None,
            json!({}),
            StatusCode::OK
        ),
        expect(
            &app,
            "POST",
            "/api/auth/demo",
            None,
            json!({}),
            StatusCode::OK
        )
    );
    assert_eq!(first["user"]["id"], second["user"]["id"]);
    assert_eq!(first["user"]["role"], "user");
    assert_eq!(first["user"]["username"], "demo_user");
    assert_ne!(first["token"], second["token"]);
    let token = first["token"].as_str().unwrap();
    expect(
        &app,
        "POST",
        "/api/communities/demo-topic/open",
        None,
        json!({}),
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let (first_open, second_open) = tokio::join!(
        expect(
            &app,
            "POST",
            "/api/communities/demo-topic/open",
            Some(token),
            json!({}),
            StatusCode::OK
        ),
        expect(
            &app,
            "POST",
            "/api/communities/demo-topic/open",
            Some(token),
            json!({}),
            StatusCode::OK
        )
    );
    assert_eq!(first_open["id"], second_open["id"]);
    assert_eq!(first_open["name"], "Demo Topic");
    let id = first_open["id"].as_str().unwrap();
    let by_id = expect(
        &app,
        "POST",
        &format!("/api/communities/{id}/open"),
        Some(token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(by_id["id"], first_open["id"]);
    let channels = expect(
        &app,
        "GET",
        &format!("/api/communities/{id}/channels"),
        Some(token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(channels.as_array().unwrap().len(), 2);
    let channel_ids: Vec<_> = channels
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].clone())
        .collect();
    for channel in channels.as_array().unwrap() {
        let channel_id = channel["id"].as_str().unwrap();
        let thread = expect(
            &app,
            "POST",
            &format!("/api/channels/{channel_id}/threads"),
            Some(token),
            json!({"title":"Demo message", "body":"Persistent community message"}),
            StatusCode::CREATED,
        )
        .await;
        assert_eq!(thread["author_id"], first["user"]["id"]);
        let thread_id = thread["id"].as_str().unwrap();
        let reply = expect(
            &app,
            "POST",
            &format!("/api/threads/{thread_id}/comments"),
            Some(token),
            json!({"body":"A demo reply"}),
            StatusCode::CREATED,
        )
        .await;
        assert_eq!(reply["author_id"], first["user"]["id"]);
        let another_token = second["token"].as_str().unwrap();
        let messages = expect(
            &app,
            "GET",
            &format!("/api/channels/{channel_id}/threads"),
            Some(another_token),
            json!(null),
            StatusCode::OK,
        )
        .await;
        assert_eq!(messages[0]["body"], "Persistent community message");
        let replies = expect(
            &app,
            "GET",
            &format!("/api/threads/{thread_id}/comments"),
            Some(another_token),
            json!(null),
            StatusCode::OK,
        )
        .await;
        assert_eq!(replies[0]["body"], "A demo reply");
    }
    // Another ordinary user is automatically joined on access, without changing the creator.
    let (_, other_token) = register(&app, "visitor").await;
    let reopened = expect(
        &app,
        "POST",
        &format!("/api/communities/{id}/open"),
        Some(&other_token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(reopened["created_by"], first["user"]["id"]);
    let reopened_channels = expect(
        &app,
        "GET",
        &format!("/api/communities/{id}/channels"),
        Some(&other_token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        channel_ids,
        reopened_channels
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].clone())
            .collect::<Vec<_>>()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM communities")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM community_members")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
    // The public demo endpoint must never issue a professional-role session.
    sqlx::query("UPDATE users SET role = 'scientist' WHERE username = 'demo_user'")
        .execute(&pool)
        .await
        .unwrap();
    expect(
        &app,
        "POST",
        "/api/auth/demo",
        None,
        json!({}),
        StatusCode::FORBIDDEN,
    )
    .await;
}

#[tokio::test]
async fn community_open_supports_uuid_short_ids_and_validates_keys() {
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app(pool.clone());
    let (_, token) = register(&app, "opener").await;
    let id = Uuid::new_v4();
    expect(
        &app,
        "GET",
        &format!("/api/communities/{id}"),
        Some(&token),
        json!(null),
        StatusCode::NOT_FOUND,
    )
    .await;
    let created = expect(
        &app,
        "POST",
        &format!("/api/communities/{id}/open"),
        Some(&token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(created["id"], id.to_string());
    let short = expect(
        &app,
        "POST",
        "/api/communities/1/open",
        Some(&token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(short["slug"], "community-1");
    let canonical = expect(
        &app,
        "POST",
        "/api/communities/community-1/open",
        Some(&token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(canonical["id"], short["id"]);
    let mixed = expect(
        &app,
        "POST",
        "/api/communities/Demo-Topic/open",
        Some(&token),
        json!({}),
        StatusCode::OK,
    )
    .await;
    assert_eq!(mixed["slug"], "demo-topic");
    for key in [
        "bad--slug",
        "-bad",
        "bad_thing",
        "%20",
        "%3Cscript%3E",
        &"a".repeat(81),
    ] {
        expect(
            &app,
            "POST",
            &format!("/api/communities/{key}/open"),
            Some(&token),
            json!({}),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM communities")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn thread_search_is_paginated_literal_and_scoped_to_members() {
    let pool = backend::connect("sqlite::memory:").await.unwrap();
    backend::migrate(&pool).await.unwrap();
    let app = backend::app(pool);
    let (_, token) = register(&app, "searcher").await;
    let (_, announcements, discussions) = community(&app, &token, "search-community").await;
    for (channel, title, body) in [
        (&discussions, "My experience", "A helpful resource"),
        (&discussions, "Resource collection", "Useful links"),
        (&discussions, "100% literal", "Underscore _"),
        (
            &announcements,
            "Resource announcement",
            "Should not match discussions",
        ),
    ] {
        expect(
            &app,
            "POST",
            &format!("/api/channels/{channel}/threads"),
            Some(&token),
            json!({"title":title,"body":body}),
            StatusCode::CREATED,
        )
        .await;
    }
    let path = format!("/api/channels/{discussions}/threads");
    let matches = expect(
        &app,
        "GET",
        &format!("{path}?q=RESOURCE"),
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(matches.as_array().unwrap().len(), 2);
    let first = expect(
        &app,
        "GET",
        &format!("{path}?q=resource&limit=1&offset=0"),
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    let second = expect(
        &app,
        "GET",
        &format!("{path}?q=resource&limit=1&offset=1"),
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(first[0]["id"], matches[0]["id"]);
    assert_eq!(second[0]["id"], matches[1]["id"]);
    let literal = expect(
        &app,
        "GET",
        &format!("{path}?q=%25"),
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert_eq!(literal.as_array().unwrap().len(), 1);
    assert_eq!(literal[0]["title"], "100% literal");
    let empty = expect(
        &app,
        "GET",
        &format!("{path}?q=missing"),
        Some(&token),
        json!(null),
        StatusCode::OK,
    )
    .await;
    assert!(empty.as_array().unwrap().is_empty());
    let (_, stranger) = register(&app, "stranger").await;
    expect(
        &app,
        "GET",
        &format!("{path}?q=resource"),
        Some(&stranger),
        json!(null),
        StatusCode::FORBIDDEN,
    )
    .await;
    expect(
        &app,
        "GET",
        &format!("{path}?q={}", "a".repeat(201)),
        Some(&token),
        json!(null),
        StatusCode::BAD_REQUEST,
    )
    .await;
}
