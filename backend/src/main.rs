use axum :: {
    Json , Router , extract::{Path,State},
    http:: StatusCode,
    routing::{get,post},
}:

use sqlx::PgPool;
use tower_http::cors::CorsLayer;



mod follows;
mod tweets;
mod users;

$[tokio::main]
async fn main (){

    dotenvy::dotenv().ok();
    let database_url =std::env::var("database_url").expect("Database url must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to database")

    sqlx::migrate("./migrations").run(&pool).await.expect("Failed to run migration");

    let app = Router::new().route("/users",get(get_users).
    post(create_user)).
    route("/users/{username}",get(get_users)).
    route("users/{username}/tweets",get(get_user_tweet)).
    route("tweets",get(get_tweets).post(create_tweet)).
    route("tweets/{id}",axum::routing::delete(delete_tweet)).
    route("tweets/feed/{user_id}",get(get_feed)).
    route("follows/toggle",post(toggle_follow)).
    layer(CorsLayer::permissive()).
    with_state(pool);


let addr = "0.0.0.0:3000"
println!("server running on http://{addr} ")
let listen = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listen,app).await.unwrap();







}

