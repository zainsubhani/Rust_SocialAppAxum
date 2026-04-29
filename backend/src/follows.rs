use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ToggleFollow {
    pub follower_id: Uuid,
    pub following_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct FollowResult {
    pub is_following: bool,
}

pub async fn toggle(pool: &PgPool, input: ToggleFollow) -> Result<FollowResult, sqlx::Error> {
    let is_following = sqlx::query_scalar!(
        r#"
        WITH deleted AS (
            DELETE FROM follows
            WHERE follower_id = $1 AND following_id = $2
            RETURNING 1
        ),
        inserted AS (
            INSERT INTO follows (follower_id, following_id)
            SELECT $1, $2
            WHERE NOT EXISTS (SELECT 1 FROM deleted)
            RETURNING 1
        )
        SELECT EXISTS (SELECT 1 FROM inserted) AS "is_following!"
        "#,
        input.follower_id,
        input.following_id
    )
    .fetch_one(pool)
    .await?;

    Ok(FollowResult { is_following })
}
