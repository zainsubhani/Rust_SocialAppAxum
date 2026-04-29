use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Tweet {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TweetWithAuthor {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTweet {
    pub author_id: Uuid,
    pub content: String,
}

pub async fn get_all_tweets(pg: &PgPool) -> Result<Vec<TweetWithAuthor>, sqlx::Error> {
    sqlx::query_as!(
        TweetWithAuthor,
        r#"
        SELECT
            t.id,
            t.author_id,
            t.content,
            t.created_at,
            u.username AS author_username,
            u.display_name AS author_display_name,
            u.avatar_url AS author_avatar_url
        FROM tweets t
        JOIN users u ON u.id = t.author_id
        ORDER BY t.created_at DESC
        "#
    )
    .fetch_all(pg)
    .await
}

pub async fn get_feed(pg: &PgPool, user_id: Uuid) -> Result<Vec<TweetWithAuthor>, sqlx::Error> {
    sqlx::query_as!(
        TweetWithAuthor,
        r#"
        SELECT
            t.id,
            t.author_id,
            t.content,
            t.created_at,
            u.username AS author_username,
            u.display_name AS author_display_name,
            u.avatar_url AS author_avatar_url
        FROM tweets t
        JOIN users u ON u.id = t.author_id
        JOIN follows f ON f.following_id = t.author_id
        WHERE f.follower_id = $1
        ORDER BY t.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pg)
    .await
}

pub async fn create(pg: &PgPool, input: CreateTweet) -> Result<TweetWithAuthor, sqlx::Error> {
    sqlx::query_as!(
        TweetWithAuthor,
        r#"
        WITH inserted AS (
            INSERT INTO tweets (author_id, content)
            VALUES ($1, $2)
            RETURNING *
        )
        SELECT
            i.id,
            i.author_id,
            i.content,
            i.created_at,
            u.username AS author_username,
            u.display_name AS author_display_name,
            u.avatar_url AS author_avatar_url
        FROM inserted i
        JOIN users u ON u.id = i.author_id
        "#,
        input.author_id,
        input.content
    )
    .fetch_one(pg)
    .await
}

pub async fn delete(pg: &PgPool, tweet_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM tweets WHERE id = $1", tweet_id)
        .execute(pg)
        .await?;

    Ok(result.rows_affected())
}
