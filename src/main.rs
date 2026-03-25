mod adapters;
mod application;
mod messaging;

use adapters::downstream::db::ComponentRepository;
use application::ComponentService;
use db_utils::init_database;
use messaging::{ComponentCreatedEvent, ComponentDTO, ComponentDeletedEvent, Publisher};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

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

    let pool = init_database().await?; 

    let repo = ComponentRepository::new(pool);
    let service = ComponentService::new(repo, "process");
    let created = service.create_component("-300.779", "Door Lock").await?;
    println!("ComponentService created component: {:?}", created);

    Ok(())
}


