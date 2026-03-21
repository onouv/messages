use anyhow::Result;
use serde_json::Value;
use sqlx::Postgres;
use uuid::Uuid;

/// Appends one outbox row within an already-open transaction.
///
/// Accepts a mutable reference to the transaction so the caller keeps ownership
/// and can commit (or roll back) after all domain inserts are done.
pub async fn insert_outbox_event(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    aggregate_type: &str,
    aggregate_id: &str,
    event_type: &str,
    view_id: &str,
    payload: Value,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, view_id, payload)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(event_type)
    .bind(view_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
