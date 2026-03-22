use anyhow::{Context, Result};
use async_nats::jetstream;
use dotenv::dotenv;
use serde_json::{Value, json};
use sqlx::{PgPool, Row, postgres::PgListener};
use tokio::time::{Duration, interval};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Config {
    database_url: String,
    nats_url: String,
    listen_channel: String,
    subject_prefix: String,
    batch_size: i64,
    fallback_poll_ms: u64,
}

#[derive(Debug)]
struct OutboxRow {
    id: Uuid,
    aggregate_type: String,
    aggregate_id: String,
    event_type: String,
    view_id: String,
    payload: Option<Value>,
}

impl Config {
    fn build_database_url_from_env() -> String {
        let db_type = std::env::var("DB_TYPE").unwrap_or_else(|_| "postgres".to_string());
        let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "fscl".to_string());
        let db_password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "fscl".to_string());
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "process".to_string());

        format!(
            "{}://{}:{}@{}:{}/{}",
            db_type, db_user, db_password, db_host, db_port, db_name
        )
    }

    fn from_env() -> Result<Self> {
        let database_url = Self::build_database_url_from_env();

        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
        let listen_channel = std::env::var("OUTBOX_NOTIFY_CHANNEL").unwrap_or_else(|_| "outbox_new".to_string());
        let subject_prefix = std::env::var("OUTBOX_SUBJECT_PREFIX").unwrap_or_else(|_| "events".to_string());

        let batch_size = std::env::var("OUTBOX_BATCH_SIZE")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(100);

        let fallback_poll_ms = std::env::var("OUTBOX_FALLBACK_POLL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30_000);

        Ok(Self {
            database_url,
            nats_url,
            listen_channel,
            subject_prefix,
            batch_size,
            fallback_poll_ms,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "outbox_relay=info,info".into()),
        )
        .init();

    let cfg = Config::from_env()?;
    info!("starting outbox-relay");

    let pool = PgPool::connect(&cfg.database_url)
        .await
        .context("failed to connect to postgres")?;

    let nats = async_nats::connect(&cfg.nats_url)
        .await
        .with_context(|| format!("failed to connect to NATS at {}", cfg.nats_url))?;
    let jetstream = jetstream::new(nats);

    let mut listener = PgListener::connect(&cfg.database_url)
        .await
        .context("failed to create LISTEN connection")?;
    listener
        .listen(&cfg.listen_channel)
        .await
        .with_context(|| format!("failed to LISTEN on {}", cfg.listen_channel))?;

    // Process any rows that were pending before the relay started.
    let startup_processed = drain_outbox(&pool, &jetstream, &cfg).await?;
    if startup_processed > 0 {
        info!(processed = startup_processed, "processed pending rows on startup");
    }

    let mut fallback_tick = interval(Duration::from_millis(cfg.fallback_poll_ms));
    info!(
        channel = cfg.listen_channel,
        poll_ms = cfg.fallback_poll_ms,
        "relay is ready"
    );

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received ctrl-c, shutting down");
                break;
            }
            _ = fallback_tick.tick() => {
                let processed = drain_outbox(&pool, &jetstream, &cfg).await?;
                if processed > 0 {
                    info!(processed, "fallback poll processed rows");
                }
            }
            recv = listener.recv() => {
                match recv {
                    Ok(notification) => {
                        info!(payload = notification.payload(), "received outbox notification");
                        let processed = drain_outbox(&pool, &jetstream, &cfg).await?;
                        info!(processed, "notify-triggered batch done");
                    }
                    Err(err) => {
                        warn!(error = %err, "LISTEN receive failed; fallback polling keeps progress");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn drain_outbox(
    pool: &PgPool,
    jetstream: &jetstream::Context,
    cfg: &Config,
) -> Result<usize> {
    let mut tx = pool.begin().await?;

    let rows = sqlx::query(
        r#"
        SELECT id, aggregate_type, aggregate_id, event_type, view_id, payload
        FROM outbox
        WHERE published_at IS NULL
        ORDER BY occurred_at
        FOR UPDATE SKIP LOCKED
        LIMIT $1
        "#,
    )
    .bind(cfg.batch_size)
    .fetch_all(&mut *tx)
    .await?;

    if rows.is_empty() {
        tx.commit().await?;
        return Ok(0);
    }

    let mut published = 0usize;

    for row in rows {
        let outbox = OutboxRow {
            id: row.try_get("id")?,
            aggregate_type: row.try_get("aggregate_type")?,
            aggregate_id: row.try_get("aggregate_id")?,
            event_type: row.try_get("event_type")?,
            view_id: row.try_get("view_id")?,
            payload: row.try_get("payload")?,
        };

        let subject = format!(
            "{}.{}.{}",
            cfg.subject_prefix, outbox.aggregate_type, outbox.event_type
        );

        // Keep the method-2 envelope format when relaying to JetStream.
        let event = json!({
            "id": outbox.id.to_string(),
            "event_type": outbox.event_type,
            "aggregate_type": outbox.aggregate_type,
            "aggregate_id": outbox.aggregate_id,
            "view_id": outbox.view_id,
            "payload": outbox.payload,
        });

        let payload = serde_json::to_vec(&event)?;

        match jetstream.publish(subject, payload.into()).await {
            Ok(pub_ack_future) => match pub_ack_future.await {
                Ok(_ack) => {
                    sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
                        .bind(outbox.id)
                        .execute(&mut *tx)
                        .await?;
                    published += 1;
                }
                Err(err) => {
                    track_failure(&mut tx, outbox.id, &format!("jetstream ack error: {err}"))
                        .await?;
                    error!(id = %outbox.id, error = %err, "publish ack failed");
                }
            },
            Err(err) => {
                track_failure(&mut tx, outbox.id, &format!("jetstream publish error: {err}"))
                    .await?;
                error!(id = %outbox.id, error = %err, "publish failed");
            }
        }
    }

    tx.commit().await?;
    Ok(published)
}

async fn track_failure(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: Uuid, err: &str) -> Result<()> {
    sqlx::query(
        "UPDATE outbox SET attempts = attempts + 1, last_error = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(err)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
