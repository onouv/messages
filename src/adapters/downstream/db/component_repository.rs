use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::insert_outbox_event;
use crate::messaging::{ComponentCreatedEvent, ComponentDTO};

pub struct ComponentRepository {
    pool: PgPool,
    view_name: String,
}

impl ComponentRepository {
    pub fn new(pool: PgPool, view_name: &str) -> Self {
        Self {
            pool,
            view_name: view_name.to_string(),
        }
    }

    /// Inserts a component row and appends an outbox event, all in one transaction.
    pub async fn create(
        &self,
        component: &ComponentDTO,
        event: ComponentCreatedEvent,
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

        // Note: triggering the publishing of events should be responsibility 
        // of the application layer, for simplicity we do it here.
        insert_outbox_event(
            &mut tx,
            Uuid::new_v4(),
            "component",
            component.id(),
            &self.view_name,
            event.to_event(),
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
