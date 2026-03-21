mod adapters;
mod application;
mod messaging;

use adapters::downstream::db::ComponentRepository;
use application::ComponentService;
use messaging::{ComponentCreatedEvent, ComponentDTO, ComponentDeletedEvent, Publisher};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    demonstrate_event_messaging();

    if let Err(e) = demonstrate_component_service().await {
        eprintln!("component service demo failed: {e}");
    }
}

fn demonstrate_event_messaging() {
    println!("demonstrating event messaging: Json payloads");
    let view = "safety";
    let component = ComponentDTO::new("-100.001", "Door Lock", None);
    let created = match ComponentCreatedEvent::to_event(component, view) {
        Ok(e) => e,
        Err(e) => {
            println!("ERROR creating ComponentCreatedEvent: {:?}", e);
            return;
        }
    };
    let deleted = ComponentDeletedEvent::to_event("-200.021".to_string(), view);

    let _ = Publisher::publish(created);
    let _ = Publisher::publish(deleted);
}

async fn demonstrate_component_service() -> anyhow::Result<()> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            println!("DATABASE_URL not set, skipping ComponentService outbox demo");
            return Ok(());
        }
    };

    let pool = PgPool::connect(&database_url).await?;

    // Sprint convenience: keep demo schema creation local to this executable.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS components (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let repo = ComponentRepository::new(pool);
    let service = ComponentService::new(repo, "process");
    let created = service.create_component("-300.777", "Access Sensor").await?;
    println!("ComponentService created component: {:?}", created);

    Ok(())
}

