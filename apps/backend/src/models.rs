use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    User,
    Scientist,
    PharmaScout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "channel_kind", rename_all = "snake_case")]
pub enum ChannelKind {
    Discussion,
    Announcement,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Community {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Membership {
    pub community_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub community_id: Uuid,
    pub slug: String,
    pub name: String,
    pub kind: ChannelKind,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub community_id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub community_id: Uuid,
    pub thread_id: Uuid,
    pub author_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct DmConversation {
    pub id: Uuid,
    pub user_low_id: Uuid,
    pub user_high_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct DmMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}
