//! The HTTP endpoints (also called "handlers").

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::hash::{DEFAULT_MAX_DISTANCE, MAX_DISTANCE, normalize_hash};

/// How many matches a search returns at most.
const SEARCH_LIMIT: i64 = 10;

// ---------- GET /health ----------

/// "Are you alive?" Also checks the database answers.
pub async fn health(State(pool): State<PgPool>) -> Result<Json<Value>, ApiError> {
    sqlx::query("SELECT 1").execute(&pool).await?;
    Ok(Json(json!({ "status": "ok", "database": "ok" })))
}

// ---------- POST /api/hashes ----------

#[derive(Deserialize)]
pub struct SubmitHash {
    hash: String,
}

#[derive(Serialize)]
pub struct Submitted {
    case_id: Uuid,
    /// Shown once. The user needs it to withdraw the case later.
    secret_key: String,
    created_at: DateTime<Utc>,
}

/// Save a fingerprint. Returns a case ID plus a secret key.
pub async fn submit_hash(
    State(pool): State<PgPool>,
    Json(body): Json<SubmitHash>,
) -> Result<(StatusCode, Json<Submitted>), ApiError> {
    let hash = normalize_hash(&body.hash).map_err(ApiError::BadRequest)?;

    let case_id = Uuid::new_v4();
    let secret_key = new_secret_key();

    let created_at: DateTime<Utc> = sqlx::query_scalar(
        r#"
        INSERT INTO hashes (case_id, hash_hex, hash_bits, secret_hash)
        VALUES ($1, $2, ('x' || $2)::bit(256), $3)
        RETURNING created_at
        "#,
    )
    .bind(case_id)
    .bind(&hash)
    .bind(sha256_hex(&secret_key))
    .fetch_one(&pool)
    .await?;

    tracing::info!(%case_id, "new case submitted");
    Ok((
        StatusCode::CREATED,
        Json(Submitted { case_id, secret_key, created_at }),
    ))
}

// ---------- POST /api/hashes/search ----------

#[derive(Deserialize)]
pub struct SearchQuery {
    hash: String,
    /// Optional. How many bits may differ and still count as a match.
    max_distance: Option<i32>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Match {
    case_id: Uuid,
    distance: i32,
}

#[derive(Serialize)]
pub struct SearchResult {
    matched: bool,
    max_distance: i32,
    matches: Vec<Match>,
}

/// "Have we seen this?" Finds stored hashes that are the same or almost the same.
pub async fn search_hash(
    State(pool): State<PgPool>,
    Json(body): Json<SearchQuery>,
) -> Result<Json<SearchResult>, ApiError> {
    let hash = normalize_hash(&body.hash).map_err(ApiError::BadRequest)?;
    let max_distance = body.max_distance.unwrap_or(DEFAULT_MAX_DISTANCE);
    if !(0..=MAX_DISTANCE).contains(&max_distance) {
        return Err(ApiError::BadRequest(format!(
            "max_distance must be between 0 and {MAX_DISTANCE}"
        )));
    }

    // `#` is XOR on bit strings: it leaves a 1 wherever the two hashes differ.
    // bit_count() then counts those 1s, which is the Hamming distance.
    let matches: Vec<Match> = sqlx::query_as(
        r#"
        SELECT case_id, distance FROM (
            SELECT case_id,
                   bit_count(hash_bits # ('x' || $1)::bit(256))::int4 AS distance
            FROM hashes
        ) scored
        WHERE distance <= $2
        ORDER BY distance ASC
        LIMIT $3
        "#,
    )
    .bind(&hash)
    .bind(max_distance)
    .bind(SEARCH_LIMIT)
    .fetch_all(&pool)
    .await?;

    Ok(Json(SearchResult {
        matched: !matches.is_empty(),
        max_distance,
        matches,
    }))
}

// ---------- DELETE /api/hashes/{case_id} ----------

/// Withdraw a case. Needs the secret key in the `x-secret-key` header.
pub async fn withdraw_hash(
    State(pool): State<PgPool>,
    Path(case_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let key = headers
        .get("x-secret-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::BadRequest("missing x-secret-key header".into()))?;

    let deleted = sqlx::query("DELETE FROM hashes WHERE case_id = $1 AND secret_hash = $2")
        .bind(case_id)
        .bind(sha256_hex(key))
        .execute(&pool)
        .await?
        .rows_affected();

    if deleted == 0 {
        // Same answer for "no such case" and "wrong key", so nobody can probe for valid IDs.
        return Err(ApiError::NotFound("case not found or wrong secret key".into()));
    }
    tracing::info!(%case_id, "case withdrawn");
    Ok(StatusCode::NO_CONTENT)
}

// ---------- small helpers ----------

/// 32 random bytes from the OS's secure random source, as 64 hex characters.
fn new_secret_key() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// We store only this fingerprint of the secret key, like a password hash.
fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}
