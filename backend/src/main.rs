mod follows;
mod tweets;
mod users;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use dotenv::dotenv;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, net::SocketAddr};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Debug)]
struct ApiError(sqlx::Error);

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.0.to_string()).into_response()
    }
}

#[derive(Debug, Serialize)]
struct DeleteResult {
    deleted: u64,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{username}", get(get_user_by_username))
        .route("/tweets", get(list_tweets).post(create_tweet))
        .route("/tweets/{tweet_id}", delete(delete_tweet))
        .route("/feed/{user_id}", get(get_feed))
        .route("/follows/toggle", post(toggle_follow))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server port");

    println!("listening on http://{addr}");

    axum::serve(listener, app).await.expect("server failed");
}

async fn list_users(State(pool): State<PgPool>) -> Result<Json<Vec<users::User>>, ApiError> {
    let users = users::get_all(&pool).await?;
    Ok(Json(users))
}

async fn get_user_by_username(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Response, ApiError> {
    match users::get_by_username(&pool, &username).await? {
        Some(user) => Ok(Json(user).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(input): Json<users::CreateUser>,
) -> Result<Json<users::User>, ApiError> {
    let user = users::create(&pool, input).await?;
    Ok(Json(user))
}

async fn list_tweets(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<tweets::TweetWithAuthor>>, ApiError> {
    let tweets = tweets::get_all_tweets(&pool).await?;
    Ok(Json(tweets))
}

async fn get_feed(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<tweets::TweetWithAuthor>>, ApiError> {
    let tweets = tweets::get_feed(&pool, user_id).await?;
    Ok(Json(tweets))
}

async fn create_tweet(
    State(pool): State<PgPool>,
    Json(input): Json<tweets::CreateTweet>,
) -> Result<Json<tweets::TweetWithAuthor>, ApiError> {
    let tweet = tweets::create(&pool, input).await?;
    Ok(Json(tweet))
}

async fn delete_tweet(
    State(pool): State<PgPool>,
    Path(tweet_id): Path<Uuid>,
) -> Result<Json<DeleteResult>, ApiError> {
    let deleted = tweets::delete(&pool, tweet_id).await?;
    Ok(Json(DeleteResult { deleted }))
}

async fn toggle_follow(
    State(pool): State<PgPool>,
    Json(input): Json<follows::ToggleFollow>,
) -> Result<Json<follows::FollowResult>, ApiError> {
    let result = follows::toggle(&pool, input).await?;
    Ok(Json(result))
}
