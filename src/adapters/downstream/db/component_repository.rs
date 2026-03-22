use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::messaging::ComponentDTO;
use super::insert_outbox_event;

pub struct ComponentRepository {
    pool: PgPool,
}

impl ComponentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inserts a component row and appends an outbox event, all in one transaction.
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

        // Note: triggering the publishing of events should be responsibility of the application layer, for simplicity we do it here.
        insert_outbox_event(&mut tx, outbox_id, "component", component.id(), event_type, view_id, event_payload).await?;

        tx.commit().await?;

        Ok(())
    }
}

