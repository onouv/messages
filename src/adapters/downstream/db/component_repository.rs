use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::messaging::ComponentDTO;

pub struct ComponentRepository {
    pool: PgPool,
}

impl ComponentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inserts a component row and an outbox row in a single transaction.
    /// The caller is responsible for building the event payload before calling this.
    pub async fn create(
        &self,
        component: &ComponentDTO,
        outbox_id: Uuid,
        event_type: &str,
        view_id: &str,
        event_payload: Value,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO components (id, name)
            VALUES ($1, $2)
            "#,
        )
        .bind(component.id())
        .bind(component.name())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, view_id, payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(outbox_id)
        .bind("component")
        .bind(component.id())
        .bind(event_type)
        .bind(view_id)
        .bind(event_payload)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
