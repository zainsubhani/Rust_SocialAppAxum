use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub created_at: OffsetDateTime,
}
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub display_name: String,
}

pub async fn get_all(pg: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users ORDER BY username")
        .fetch_all(pg)
        .await
}

pub async fn get_by_username(pg: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
        .fetch_optional(pg)
        .await
}

pub async fn create(pg: &PgPool, input: CreateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "
        INSERT INTO users (username, display_name)
        VALUES ($1, $2)
        RETURNING *
        ",
        input.username,
        input.display_name
    )
    .fetch_one(pg)
    .await
}
