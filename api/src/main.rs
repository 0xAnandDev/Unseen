//! Unseen API: stores image fingerprints (never images) and finds near-duplicates.

mod error;
mod hash;
mod routes;

use std::{net::SocketAddr, time::Duration};

use axum::{
    Router,
    routing::{delete, get, post},
};
use sqlx::postgres::PgPoolOptions;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read settings from a .env file if there is one (handy in development).
    dotenvy::dotenv().ok();

    // Logging. Set RUST_LOG=debug for more detail.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "unseen_api=info,tower_http=debug".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    // A pool keeps a few database connections open and shares them between requests.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    // Create or update the tables from the files in ./migrations.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Rate limiting per IP address: a burst of up to 10 requests,
    // then one more allowed every 500 ms. Extra requests get HTTP 429.
    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(500)
        .burst_size(10)
        .finish()
        .expect("valid rate limit config");

    // Clean old entries out of the rate limiter's memory once a minute.
    let limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            limiter.retain_recent();
        }
    });

    // The API routes, with rate limiting.
    let api = Router::new()
        .route("/hashes", post(routes::submit_hash))
        .route("/hashes/search", post(routes::search_hash))
        .route("/hashes/{case_id}", delete(routes::withdraw_hash))
        .layer(GovernorLayer::new(governor_conf));

    let app = Router::new()
        .route("/health", get(routes::health))
        .nest("/api", api)
        .layer(TraceLayer::new_for_http()) // logs every request
        .layer(CorsLayer::permissive()) // lets the React dev server call us (tighten before going live)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Unseen API listening on http://localhost:{port}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    // `with_connect_info` gives the rate limiter each caller's IP address.
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
